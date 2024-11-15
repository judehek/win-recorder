use crate::error::RecorderError;
use log::{error, info};
use std::ptr;
use windows::core::{Array, ComInterface, Result, GUID};
use windows::Win32::Media::MediaFoundation::*;

pub fn check_encoder_support() -> Result<Vec<String>> {
    let mut encoders = Vec::new();

    unsafe {
        // Initialize MF if not already initialized
        MFStartup(MF_VERSION, MFSTARTUP_FULL)?;

        // Enumerate transforms
        let mut data = std::ptr::null_mut();
        let mut len = 0;

        MFTEnumEx(
            MFT_CATEGORY_VIDEO_ENCODER,
            MFT_ENUM_FLAG_HARDWARE | MFT_ENUM_FLAG_SORTANDFILTER,
            None,
            None,
            &mut data,
            &mut len,
        )?;

        let activates = Array::<IMFActivate>::from_raw_parts(data as _, len);

        // Process each activate object
        for activate in activates.as_slice() {
            let activate = activate.clone().unwrap();

            // Get the friendly name using the helper function
            if let Ok(Some(name)) = get_string_attribute(
                activate.cast::<IMFAttributes>().as_ref().unwrap(),
                &MFT_FRIENDLY_NAME_Attribute,
            ) {
                info!("Found encoder: {}", name);
                encoders.push(name);
            }

            // Get additional details about the encoder
            if let Ok(transform) = activate.ActivateObject::<IMFTransform>() {
                // Get input types
                let mut input_types = Vec::new();
                let mut index = 0;
                loop {
                    let type_info = transform.GetInputAvailableType(0, index);
                    match type_info {
                        Ok(type_info) => {
                            // Get the GUID directly from the method
                            if let Ok(guid) = type_info.GetGUID(&MF_MT_SUBTYPE) {
                                input_types.push(guid);
                            }
                            index += 1;
                        }
                        Err(_) => break,
                    }
                }

                // Log supported input formats
                if !input_types.is_empty() {
                    info!("Supported input formats: {:?}", input_types);
                }
            }
        }
    }

    Ok(encoders)
}

pub fn get_string_attribute(
    attributes: &IMFAttributes,
    attribute_guid: &GUID,
) -> Result<Option<String>> {
    unsafe {
        match attributes.GetStringLength(attribute_guid) {
            Ok(mut length) => {
                let mut result = vec![0u16; (length + 1) as usize];
                attributes.GetString(attribute_guid, &mut result, Some(&mut length))?;
                result.resize(length as usize, 0);
                Ok(Some(String::from_utf16(&result).unwrap()))
            }
            Err(error) => {
                if error.code() == MF_E_ATTRIBUTENOTFOUND {
                    Ok(None)
                } else {
                    Err(error)
                }
            }
        }
    }
}

pub fn diagnose_encoding_capabilities() -> std::result::Result<(), RecorderError> {
    info!("Diagnosing encoding capabilities...");

    match check_encoder_support() {
        Ok(encoders) => {
            if encoders.is_empty() {
                error!("No hardware encoders found!");
                return Err(RecorderError::NoEncodersFound);
            }

            let has_nvidia = encoders
                .iter()
                .any(|name| name.to_lowercase().contains("nvidia"));
            let has_intel = encoders
                .iter()
                .any(|name| name.to_lowercase().contains("intel"));

            info!("Found {} encoders:", encoders.len());
            for encoder in &encoders {
                info!("  - {}", encoder);
            }

            if has_nvidia {
                info!("NVIDIA encoder detected");
            }
            if has_intel {
                info!("Intel Quick Sync encoder detected");
            }

            Ok(())
        }
        Err(e) => {
            error!("Failed to enumerate encoders: {:?}", e);
            Err(RecorderError::EncoderEnumerationFailed(e.to_string()))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encoder_enumeration() {
        let result = check_encoder_support();
        assert!(result.is_ok(), "Encoder enumeration should succeed");

        let encoders = result.unwrap();
        assert!(!encoders.is_empty(), "Should find at least one encoder");
    }
}
