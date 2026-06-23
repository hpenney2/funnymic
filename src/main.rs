use std::{collections::HashMap, error::Error, ffi::CString};

use inquire::Select;
use pulseaudio::ClientError::ServerError;
use pulseaudio::protocol::PulseError::NoEntity;
use rdev::{Event, EventType, Key, listen};

const CONTROLLER_KEY: Key = Key::F4;

// TODO: we can probably switch String to CString since we usually need it to be one,
// though i'm not sure when we add Windows support
pub struct InputSwapper {
    pulse_client: pulseaudio::Client,
    original_source: CString,
    pub swap_to: CString,
    active: bool,
}
impl InputSwapper {
    pub fn new(pulse_client: pulseaudio::Client, swap_to: CString) -> Self {
        InputSwapper {
            pulse_client,
            original_source: c"".to_owned(),
            swap_to,
            active: false,
        }
    }

    pub async fn swap_on(&mut self) {
        if self.active {
            return;
        }

        println!("on...");

        // dear lord
        self.original_source = self
            .pulse_client
            .server_info()
            .await
            .expect("failed to get defaults")
            .default_source_name
            .unwrap();

        self.switch_source(self.swap_to.clone()).await;

        self.active = true;
        println!("on!")
    }

    pub async fn swap_off(&mut self) {
        if !self.active {
            return;
        }

        println!("off...");

        self.switch_source(self.original_source.clone()).await;

        self.active = false;
        println!("off!")
    }

    async fn switch_source(&self, source: CString) {
        // TODO: not crash when failing to swap? (consider this when implementing UI?)
        match self.pulse_client.set_default_source(source).await {
            Ok(_) => (),
            Err(ServerError(NoEntity)) => {
                panic!("failed to swap source because source was removed")
            }
            Err(err) => panic!("failed to swap source: {:?}", err),
        };
    }

    fn key_callback(&mut self, event: Event) {
        // the only reason i do this is because rdev doesn't support asnyc functions
        // and so we need a way to run something that *is* async (which we need because of PulseAudio's API)
        // from sync. HOWEVER, we can do this sync using the library's lower level API.
        // considering UI is a next step, we'll likely want async anyway and need to just rework the key listener instead

        match event.event_type {
            EventType::KeyPress(CONTROLLER_KEY) if !self.active => {
                futures::executor::block_on(self.swap_on())
            }

            EventType::KeyRelease(CONTROLLER_KEY) if self.active => {
                futures::executor::block_on(self.swap_off())
            }
            _ => (),
        };
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    println!("FUNNY MIC!");
    let client = pulseaudio::Client::from_env(c"funnymic")?;
    let mut source_map: HashMap<String, String> = HashMap::new();
    let sources = client
        .list_sources()
        .await?
        .into_iter()
        .map(|src| {
            source_map.insert(
                src.description
                    .clone()
                    .map(|x| x.into_string().unwrap())
                    .unwrap_or(format!("??? [index {}]", src.index)),
                src.name.into_string().unwrap(),
            );

            src.description
                .map(|x| x.into_string().unwrap())
                .unwrap_or(format!("??? [index {}]", src.index))
        })
        .collect();

    let funny_source = Select::new(
        "Which microphone should be swapped to when the controller key is held?",
        sources,
    )
    .prompt()?;

    // docs say that cloning the PulseAudio client is allowed!
    let mut swapper = InputSwapper::new(
        client.clone(),
        CString::new(source_map.get(&funny_source).unwrap().to_owned())?,
    );

    println!("Listening for {:?}...", CONTROLLER_KEY);
    if let Err(error) = listen(move |event| swapper.key_callback(event)) {
        eprintln!("Listening error: {:?}", error);
        panic!();
    }

    Ok(())
}
