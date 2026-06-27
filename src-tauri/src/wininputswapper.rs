use std::collections::HashMap;

use com_policy_config::{IPolicyConfig, PolicyConfigClient};
use windows::core::{Result as WinResult, HSTRING, PCWSTR};
use windows::Win32::Devices::FunctionDiscovery::PKEY_Device_FriendlyName;
use windows::Win32::Media::Audio::{
    eCapture, eCommunications, IMMDeviceEnumerator, MMDeviceEnumerator, DEVICE_STATE_ACTIVE,
};
use windows::Win32::System::Com::{
    CoCreateInstance, CoInitializeEx, CoUninitialize, CLSCTX_ALL, COINIT_APARTMENTTHREADED,
    STGM_READ,
};

pub struct WinInputSwapper {
    original_source: String,
    pub swap_to: String,
}
impl WinInputSwapper {
    pub fn new(swap_to: String) -> Self {
        unsafe {
            CoInitializeEx(None, COINIT_APARTMENTTHREADED)
                .ok()
                .expect("failed to initialize COM");
        }
        WinInputSwapper {
            original_source: String::new(),
            swap_to,
        }
    }

    pub fn swap_on(&mut self) -> WinResult<()> {
        if self.swap_to.is_empty() {
            eprintln!("no swap device set");
            return Ok(());
        }

        println!("on...");

        // dear lord
        self.original_source = self.get_default_device()?;

        self.switch_source(&self.swap_to.clone())?;

        println!("on!");
        Ok(())
    }

    pub fn swap_off(&mut self) -> WinResult<()> {
        println!("off...");

        self.switch_source(&self.original_source.clone())?;

        println!("off!");
        Ok(())
    }

    fn switch_source(&mut self, source: &str) -> WinResult<()> {
        // this hstring needs to be kept seperate so we don't have a dangling pointer!
        let hstring = HSTRING::from(source);
        let device_id = PCWSTR(hstring.as_ptr());

        unsafe {
            // THIS IS AN UNDOCUMENTED WINDOWS API!
            // This is being importing in from a crate, so hopefully that will be updated if it ever changes.
            let policy_config: IPolicyConfig =
                CoCreateInstance(&PolicyConfigClient, None, CLSCTX_ALL)?;
            policy_config.SetDefaultEndpoint(device_id, eCommunications)?;
        };

        Ok(())
    }

    fn get_default_device(&self) -> WinResult<String> {
        unsafe {
            let enumerator: IMMDeviceEnumerator =
                CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_ALL)?;
            Ok(enumerator
                .GetDefaultAudioEndpoint(eCapture, eCommunications)?
                .GetId()?
                .to_string()?)
        }
    }

    pub fn get_sources(&self) -> WinResult<HashMap<String, String>> {
        let mut devices = HashMap::new();

        unsafe {
            let enumerator: IMMDeviceEnumerator =
                CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_ALL)?;
            let device_collection = enumerator.EnumAudioEndpoints(eCapture, DEVICE_STATE_ACTIVE)?;

            for i in 0..device_collection.GetCount()? {
                let device = device_collection.Item(i)?;
                let props = device.OpenPropertyStore(STGM_READ)?;
                let friendly_name = props
                    .GetValue(&PKEY_Device_FriendlyName)?
                    .Anonymous
                    .Anonymous
                    .Anonymous
                    .pwszVal // as LPWSTR, i.e. string of 16-bit Unicode characters
                    .to_string()?;

                devices.insert(device.GetId()?.to_string()?, friendly_name);
            }
        };

        Ok(devices)
    }
}

impl Drop for WinInputSwapper {
    fn drop(&mut self) {
        if !self.original_source.is_empty() {
            let _ = self.swap_off();
        }

        unsafe {
            CoUninitialize();
        }
    }
}
