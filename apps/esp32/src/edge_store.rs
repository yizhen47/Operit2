#![allow(non_snake_case)]

use std::sync::Mutex;

use esp_idf_svc::nvs::{EspDefaultNvs, EspDefaultNvsPartition, EspNvs};
use operit_edge_transport::{EdgePairingPersistentState, EdgePairingStore};
use operit_host_api::{HostError, HostResult};

const EDGE_PAIRING_NAMESPACE: &str = "operit_edge";
const EDGE_PAIRING_STATE_KEY: &str = "pairing_state";

/// NVS-backed storage for the small Edge pairing state.
pub struct Esp32EdgePairingStore {
    nvs: Mutex<EspDefaultNvs>,
}

impl Esp32EdgePairingStore {
    pub fn new(partition: EspDefaultNvsPartition) -> HostResult<Self> {
        let nvs = EspNvs::new(partition, EDGE_PAIRING_NAMESPACE, true)
            .map_err(|error| HostError::new(format!("edge pairing NVS: {error}")))?;
        Ok(Self { nvs: Mutex::new(nvs) })
    }
}

impl EdgePairingStore for Esp32EdgePairingStore {
    fn load(&self) -> Result<Option<EdgePairingPersistentState>, String> {
        let nvs = self.nvs.lock().map_err(|error| error.to_string())?;
        let Some(length) = nvs.blob_len(EDGE_PAIRING_STATE_KEY).map_err(|error| error.to_string())? else {
            return Ok(None);
        };
        let mut bytes = vec![0u8; length];
        let Some(bytes) = nvs
            .get_blob(EDGE_PAIRING_STATE_KEY, &mut bytes)
            .map_err(|error| error.to_string())?
        else {
            return Ok(None);
        };
        serde_json::from_slice(bytes)
            .map(Some)
            .map_err(|error| format!("decode Edge pairing NVS state: {error}"))
    }

    fn save(&self, state: &EdgePairingPersistentState) -> Result<(), String> {
        let bytes = serde_json::to_vec(state)
            .map_err(|error| format!("encode Edge pairing NVS state: {error}"))?;
        self.nvs
            .lock()
            .map_err(|error| error.to_string())?
            .set_blob(EDGE_PAIRING_STATE_KEY, &bytes)
            .map_err(|error| error.to_string())
    }
}
