use std::{collections::HashMap, error::Error, ffi::CString};

use inquire::Select;
use rdev::{Event, EventType, Key, listen};
use tokio::runtime::Handle;

const CONTROLLER_KEY: Key = Key::F4;

// TODO: we can probably switch String to CString since we usually need it to be one,
// though i'm not sure when we add Windows support
pub struct InputSwapper {
    pulse_client: pulseaudio::Client,
    original_source: String,
    pub swap_to: String,
    active: bool,
}
impl InputSwapper {
    fn new(pulse_client: pulseaudio::Client, swap_to: String) -> Self {
        InputSwapper {
            pulse_client,
            original_source: String::new(),
            swap_to,
            active: false,
        }
    }

    async fn swap_on(&mut self) {
        if self.active {
            return;
        }

        // dear lord
        self.original_source = self
            .pulse_client
            .server_info()
            .await
            .expect("failed to get defaults")
            .default_source_name
            .unwrap()
            .into_string()
            .unwrap();

        self.pulse_client
            .set_default_source(CString::new(self.swap_to.clone()).unwrap())
            .await
            .expect("failed to swap source");

        self.active = true;
        println!("on!")
    }

    async fn swap_off(&mut self) {
        if !self.active {
            return;
        }

        self.pulse_client
            .set_default_source(CString::new(self.original_source.clone()).unwrap())
            .await
            .expect("failed to swap source");

        self.active = false;
        println!("off...")
    }

    fn key_callback(&mut self, event: Event) {
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

    let mut swapper = InputSwapper::new(client, source_map.get(&funny_source).unwrap().to_owned());

    println!("Listening for {:?}...", CONTROLLER_KEY);
    if let Err(error) = listen(move |event| swapper.key_callback(event)) {
        eprintln!("Listening error: {:?}", error);
        panic!();
    }

    Ok(())
}
