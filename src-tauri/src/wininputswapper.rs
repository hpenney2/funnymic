use std::collections::HashMap;

use com_policy_config::{IPolicyConfig, PolicyConfigClient};
use windows::core::{Result as WinResult, HRESULT, HSTRING, PCWSTR};
use windows::Win32::Devices::FunctionDiscovery::PKEY_Device_FriendlyName;
use windows::Win32::Foundation::ERROR_NOT_FOUND;
use windows::Win32::Media::Audio::{
    eCapture, eCommunications, eConsole, eMultimedia, ERole, IMMDeviceEnumerator,
    MMDeviceEnumerator, DEVICE_STATE_ACTIVE,
};
use windows::Win32::System::Com::{
    CoCreateInstance, CoInitializeEx, CoUninitialize, CLSCTX_ALL, COINIT_APARTMENTTHREADED,
    STGM_READ,
};

const E_NOTFOUND: HRESULT = HRESULT::from_win32(ERROR_NOT_FOUND.0); // this should be correct? i believe E_NOTFOUND is ERROR_NOT_FOUND wrapped in an HRESULT

pub struct WinInputSwapper {
    original_source_console: Option<String>,
    original_source_mm: Option<String>,
    original_source_comms: Option<String>,
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
            original_source_console: None,
            original_source_mm: None,
            original_source_comms: None,
            swap_to,
        }
    }

    pub fn swap_on(&mut self) -> WinResult<()> {
        if self.swap_to.is_empty() {
            eprintln!("no swap device set");
            return Ok(());
        }

        println!("on...");

        self.original_source_console = self.get_default_device(eConsole)?;
        self.original_source_mm = self.get_default_device(eMultimedia)?;
        self.original_source_comms = self.get_default_device(eCommunications)?;

        self.switch_source(&self.swap_to.clone(), eCommunications)?;
        self.switch_source(&self.swap_to.clone(), eMultimedia)?;
        self.switch_source(&self.swap_to.clone(), eConsole)?;

        println!("on!");
        Ok(())
    }

    pub fn swap_off(&mut self) -> WinResult<()> {
        println!("off...");

        if let Some(device) = &self.original_source_console {
            self.switch_source(&device.clone(), eConsole)?;
        }

        if let Some(device) = &self.original_source_mm {
            self.switch_source(&device.clone(), eMultimedia)?;
        }

        if let Some(device) = &self.original_source_comms {
            self.switch_source(&device.clone(), eCommunications)?;
        }

        println!("off!");
        Ok(())
    }

    fn switch_source(&mut self, source: &str, role: ERole) -> WinResult<()> {
        // this hstring needs to be kept seperate so we don't have a dangling pointer!
        let hstring = HSTRING::from(source);
        let device_id = PCWSTR(hstring.as_ptr());

        unsafe {
            // THIS IS AN UNDOCUMENTED WINDOWS API!
            // This is being importing in from a crate, so hopefully that will be updated if it ever changes.
            let policy_config: IPolicyConfig =
                CoCreateInstance(&PolicyConfigClient, None, CLSCTX_ALL)?;
            policy_config.SetDefaultEndpoint(device_id, role)?;
        };

        Ok(())
    }

    fn get_default_device(&self, role: ERole) -> WinResult<Option<String>> {
        unsafe {
            let enumerator: IMMDeviceEnumerator =
                CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_ALL)?;
            match enumerator.GetDefaultAudioEndpoint(eCapture, role) {
                Ok(device) => Ok(Some(device.GetId()?.to_string()?)),
                Err(err) if err.code() == E_NOTFOUND => Ok(None),
                Err(err) => Err(err),
            }
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
        if !self.original_source_comms.is_none()
            || !self.original_source_mm.is_none()
            || !self.original_source_comms.is_none()
        {
            let _ = self.swap_off();
        }

        unsafe {
            CoUninitialize();
        }
    }
}
