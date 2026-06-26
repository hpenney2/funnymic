use std::{collections::HashMap, ffi::CString};

use pulseaudio::{
    protocol::PulseError::NoEntity,
    ClientError::{self, ServerError},
};

pub struct PAInputSwapper {
    pub pulse_client: pulseaudio::Client,
    original_source: CString,
    pub swap_to: CString,
}
impl PAInputSwapper {
    pub fn new(pulse_client: pulseaudio::Client) -> Self {
        PAInputSwapper {
            pulse_client,
            original_source: c"".to_owned(),
            swap_to: c"".to_owned(),
        }
    }

    pub async fn swap_on(&mut self) -> Result<(), ClientError> {
        if self.swap_to.is_empty() {
            eprintln!("no swap device set");
            return Ok(());
        }

        println!("on...");

        // dear lord
        self.original_source = self
            .pulse_client
            .server_info()
            .await?
            .default_source_name
            .unwrap();

        self.switch_source(self.swap_to.clone()).await?;

        println!("on!");
        Ok(())
    }

    pub async fn swap_off(&mut self) -> Result<(), ClientError> {
        println!("off...");

        self.switch_source(self.original_source.clone()).await?;

        println!("off!");
        Ok(())
    }

    async fn switch_source(&mut self, source: CString) -> Result<(), ClientError> {
        match self.pulse_client.set_default_source(source.clone()).await {
            Ok(_) => Ok(()),
            Err(err @ ServerError(NoEntity)) => {
                if self.swap_to == source {
                    self.swap_to = c"".to_owned(); // source isn't valid anymore, get it out of here!
                }
                Err(err)
            }
            Err(err) => Err(err),
        }
    }

    /// Returns sources as a HashMap of PulseAudio IDs (keys) to descriptions (values).
    pub async fn get_sources(&self) -> Result<HashMap<String, String>, ClientError> {
        let sources: HashMap<String, String> = self
            .pulse_client
            .list_sources()
            .await?
            .into_iter()
            .map(|src| {
                (
                    src.name.into_string().unwrap(),
                    src.description
                        .clone()
                        .map(|x| x.into_string().unwrap())
                        .unwrap_or(format!("??? [index {}]", src.index)),
                )
            })
            .collect();

        Ok(sources)
    }
}
