// Copyright (c) 2026 Randall Rosas (Slategray).
// All rights reserved.

// ===========================================================================
// KERNEL STREAMING FILTER & PIN DESCRIPTORS
// ===========================================================================

// External crates.
use wdk_sys::ntddk::*;
use wdk_sys::*;

// Local modules.
use crate::constants::*;
use crate::stream::{
    KSDATAFORMAT, KSDATARANGE, KSDATARANGE_AUDIO, KSPIN_DESCRIPTOR, PCAUTOMATION_TABLE,
    PCCONNECTION, PCFILTER_DESCRIPTOR, PCPIN_DESCRIPTOR, PCPROPERTY_ITEM, PPCPROPERTY_REQUEST,
};

// KSCOMPONENTID matching Windows SDK layout.
#[repr(C)]
#[allow(non_snake_case)]
pub struct KSCOMPONENTID {
    pub Manufacturer: GUID,
    pub Product: GUID,
    pub Component: GUID,
    pub Name: GUID,
    pub Version: u32,
    pub Revision: u32,
}

#[repr(C)]
#[derive(Copy, Clone)]
#[allow(non_snake_case)]
pub struct KSINTERFACE_STANDARD {
    pub InterfaceId: GUID,
    pub Reserved: u32,
    pub Version: u32,
}

#[repr(C)]
#[allow(non_snake_case)]
pub struct KSPROPERTY {
    pub Set: GUID,
    pub Id: u32,
    pub Flags: u32,
}

#[repr(C)]
#[allow(non_snake_case)]
pub struct KSIDENTIFIER {
    pub Set: GUID,
    pub Id: u32,
    pub Flags: u32,
}

#[repr(C)]
#[allow(non_snake_case)]
pub struct KSPROPERTY_DESCRIPTION {
    pub AccessFlags: u32,
    pub DescriptionSize: u32,
    pub PropTypeSet: KSIDENTIFIER,
    pub MembersListCount: u32,
    pub Reserved: u32,
}

#[repr(C)]
#[allow(non_snake_case)]
pub struct KSPROPERTY_MEMBERSHEADER {
    pub MembersFlags: u32,
    pub MembersSize: u32,
    pub MembersCount: u32,
    pub Flags: u32,
}

#[repr(C)]
#[allow(non_snake_case)]
pub struct KSPROPERTY_STEPPING_LONG {
    pub SteppingDelta: u32,
    pub Reserved: u32,
    pub SignedMinimum: i32,
    pub SignedMaximum: i32,
}

pub const KSPROPTYPESETID_GENERAL_PROP: GUID = GUID {
    Data1: 0x97E99BA0,
    Data2: 0xBDEA,
    Data3: 0x11CF,
    Data4: [0xA5, 0xD6, 0x28, 0xDB, 0x04, 0xC1, 0x00, 0x00],
};

const KSPROPERTY_MEMBER_STEPPEDRANGES: u32 = 2;
const VT_I4: u32 = 3;
const VT_BOOL: u32 = 11;

#[repr(C)]
#[allow(non_snake_case)]
pub struct KSJACK_DESCRIPTION {
    pub ChannelMapping: u32,
    pub Color: u32,
    pub ConnectionType: u32,
    pub GeoLocation: u32,
    pub GenLocation: u32,
    pub PortConnection: u32,
    pub IsConnected: u32,
}

#[repr(C)]
#[allow(non_snake_case)]
pub struct KSJACK_DESCRIPTION2 {
    pub DeviceStateInfo: u32,
    pub JackCapabilities: u32,
}

pub const JACK_COLOR_BLACK: u32 = 0x000000;

#[link_section = ".rdata"]
pub static KSINTERFACES: [KSIDENTIFIER; 1] = [KSIDENTIFIER {
    Set: KSINTERFACESETID_STANDARD,
    Id: 0, // KSINTERFACE_STANDARD_STREAMING
    Flags: 0,
}];

#[repr(transparent)]
pub struct SyncPtr<T>(pub *const T);
unsafe impl<T> Sync for SyncPtr<T> {}

// ===========================================================================
// Data Ranges
// ===========================================================================

#[link_section = ".rdata"]
pub static PCM_DATARANGE: KSDATARANGE_AUDIO = KSDATARANGE_AUDIO {
    DataRange: KSDATARANGE {
        FormatSize: core::mem::size_of::<KSDATARANGE_AUDIO>() as u32,
        Flags: 0,
        SampleSize: 0,
        Reserved: 0,
        MajorFormat: KSDATAFORMAT_TYPE_AUDIO,
        SubFormat: KSDATAFORMAT_SUBTYPE_PCM,
        Specifier: KSDATAFORMAT_SPECIFIER_WAVEFORMATEX,
    },
    MaximumChannels: 2,
    MinimumBitsPerSample: 8,
    MaximumBitsPerSample: 32,
    MinimumSampleFrequency: 8000,
    MaximumSampleFrequency: 192000,
};

#[link_section = ".rdata"]
pub static FLOAT_DATARANGE: KSDATARANGE_AUDIO = KSDATARANGE_AUDIO {
    DataRange: KSDATARANGE {
        FormatSize: core::mem::size_of::<KSDATARANGE_AUDIO>() as u32,
        Flags: 0,
        SampleSize: 0,
        Reserved: 0,
        MajorFormat: KSDATAFORMAT_TYPE_AUDIO,
        SubFormat: KSDATAFORMAT_SUBTYPE_IEEE_FLOAT,
        Specifier: KSDATAFORMAT_SPECIFIER_WAVEFORMATEX,
    },
    MaximumChannels: 2,
    MinimumBitsPerSample: 8,
    MaximumBitsPerSample: 32,
    MinimumSampleFrequency: 8000,
    MaximumSampleFrequency: 192000,
};

#[link_section = ".rdata"]
pub static BRIDGE_DATARANGE: KSDATARANGE = KSDATARANGE {
    FormatSize: core::mem::size_of::<KSDATARANGE>() as u32,
    Flags: 0,
    SampleSize: 0,
    Reserved: 0,
    MajorFormat: KSDATAFORMAT_TYPE_AUDIO,
    SubFormat: KSDATAFORMAT_SUBTYPE_ANALOG,
    Specifier: KSDATAFORMAT_SPECIFIER_NONE_GUID,
};

#[link_section = ".rdata"]
pub static WAVE_DATARANGES: [SyncPtr<KSDATARANGE>; 2] = [
    SyncPtr(&PCM_DATARANGE.DataRange as *const KSDATARANGE),
    SyncPtr(&FLOAT_DATARANGE.DataRange as *const KSDATARANGE),
];

#[link_section = ".rdata"]
pub static BRIDGE_DATARANGES: [SyncPtr<KSDATARANGE>; 1] =
    [SyncPtr(&BRIDGE_DATARANGE as *const KSDATARANGE)];

#[link_section = ".rdata"]
pub static GENERAL_PROPERTIES: [PCPROPERTY_ITEM; 1] = [PCPROPERTY_ITEM {
    Set: &KSPROPSETID_GENERAL as *const GUID,
    Id: KSPROPERTY_GENERAL_COMPONENTID,
    Flags: KSPROPERTY_TYPE_GET | KSPROPERTY_TYPE_BASICSUPPORT,
    Handler: Some(component_id_handler),
}];

#[allow(clippy::missing_safety_doc)]
pub unsafe extern "C" fn component_id_handler(property_request: PPCPROPERTY_REQUEST) -> NTSTATUS {
    if property_request.is_null() {
        return STATUS_INVALID_PARAMETER;
    }

    DbgPrint(c"LeylineKernel: KSPROPERTY_GENERAL_COMPONENTID\n".as_ptr());

    if ((*property_request).Verb & KSPROPERTY_TYPE_BASICSUPPORT) != 0 {
        let full_size = core::mem::size_of::<KSPROPERTY_DESCRIPTION>() as u32;
        let ulong_size = core::mem::size_of::<u32>() as u32;

        if (*property_request).ValueSize == 0 {
            (*property_request).ValueSize = full_size;
            return STATUS_BUFFER_OVERFLOW;
        }

        if (*property_request).ValueSize >= full_size {
            let desc = (*property_request).Value as *mut KSPROPERTY_DESCRIPTION;
            if !desc.is_null() {
                (*desc).AccessFlags = KSPROPERTY_TYPE_GET | KSPROPERTY_TYPE_BASICSUPPORT;
                (*desc).DescriptionSize = full_size;
                (*desc).PropTypeSet.Set = KSPROPTYPESETID_GENERAL_PROP;
                (*desc).PropTypeSet.Id = VT_I4; // Identifier isn't strictly numeric or bool but VT_I4 fallback
                (*desc).PropTypeSet.Flags = 0;
                (*desc).MembersListCount = 0;
                (*desc).Reserved = 0;
            }
            (*property_request).ValueSize = full_size;
            return STATUS_SUCCESS;
        } else if (*property_request).ValueSize >= ulong_size {
            let flags = (*property_request).Value as *mut u32;
            if !flags.is_null() {
                *flags = KSPROPERTY_TYPE_GET | KSPROPERTY_TYPE_BASICSUPPORT;
            }
            (*property_request).ValueSize = ulong_size;
            return STATUS_SUCCESS;
        }
        return STATUS_BUFFER_TOO_SMALL;
    }

    // AEB often queries this to verify the driver identity.
    // PortCls PCPROPERTY_REQUEST uses ValueSize, not ValueLength.
    if (*property_request).ValueSize == 0 {
        (*property_request).ValueSize = core::mem::size_of::<KSCOMPONENTID>() as u32;
        return STATUS_BUFFER_OVERFLOW;
    }

    if (*property_request).ValueSize < core::mem::size_of::<KSCOMPONENTID>() as u32 {
        return STATUS_BUFFER_TOO_SMALL;
    }

    let component_id = (*property_request).Value as *mut KSCOMPONENTID;
    if !component_id.is_null() {
        // Fields must be written in SDK order: Manufacturer, Product, Component, Name.
        (*component_id).Manufacturer = GUID {
            Data1: 0x534C,
            Data2: 0x4154,
            Data3: 0x4547,
            Data4: [0x52, 0x41, 0x59, 0x44, 0x45, 0x56, 0x31, 0x31],
        };
        (*component_id).Product = GUID {
            Data1: 0x4C45,
            Data2: 0x594C,
            Data3: 0x494E,
            Data4: [0x45, 0x41, 0x55, 0x44, 0x49, 0x4F, 0x31, 0x31],
        };
        (*component_id).Component = GUID {
            Data1: 0xDEADBEEF,
            Data2: 0xCAFE,
            Data3: 0xFEED,
            Data4: [0x4C, 0x45, 0x59, 0x4C, 0x49, 0x4E, 0x45, 0x31],
        };
        (*component_id).Name = GUID_NULL;
        (*component_id).Version = 1;
        (*component_id).Revision = 0;
    }

    STATUS_SUCCESS
}

#[allow(clippy::missing_safety_doc)]
pub unsafe extern "C" fn jack_description_handler(
    property_request: PPCPROPERTY_REQUEST,
) -> NTSTATUS {
    if property_request.is_null() {
        return STATUS_INVALID_PARAMETER;
    }

    // CRITICAL: The AEB's first call is always a basic-support query where
    // ValueSize == 0 and Instance may be NULL. We MUST handle this before
    // dereferencing Instance, otherwise we BSOD on the very first AEB query.
    // CRITICAL FIX: The ID is located in the PropertyItem, not the Instance.
    // PortCls passes the PropertyItem pointer in the PCPROPERTY_REQUEST.
    if (*property_request).PropertyItem.is_null() {
        DbgPrint(c"Leyline: jack_description_handler - PropertyItem is NULL\n".as_ptr());
        return STATUS_INVALID_PARAMETER;
    }

    let prop_id = (*(*property_request).PropertyItem).Id;
    let mut pin_id = !0u32;
    if (*property_request).InstanceSize >= core::mem::size_of::<u32>() as u32 {
        pin_id = *((*property_request).Instance as *const u32);
    }

    // Handle Basic Support
    if ((*property_request).Verb & KSPROPERTY_TYPE_BASICSUPPORT) != 0 {
        let full_size = core::mem::size_of::<KSPROPERTY_DESCRIPTION>() as u32;
        let ulong_size = core::mem::size_of::<u32>() as u32;

        if (*property_request).ValueSize == 0 {
            (*property_request).ValueSize = full_size;
            return STATUS_BUFFER_OVERFLOW;
        }

        if (*property_request).ValueSize >= full_size {
            let desc = (*property_request).Value as *mut KSPROPERTY_DESCRIPTION;
            if !desc.is_null() {
                (*desc).AccessFlags = KSPROPERTY_TYPE_GET | KSPROPERTY_TYPE_BASICSUPPORT;
                (*desc).DescriptionSize = full_size;
                (*desc).PropTypeSet.Set = KSPROPTYPESETID_GENERAL_PROP;
                (*desc).PropTypeSet.Id = VT_I4;
                (*desc).PropTypeSet.Flags = 0;
                (*desc).MembersListCount = 0;
                (*desc).Reserved = 0;
            }
            (*property_request).ValueSize = full_size;
            return STATUS_SUCCESS;
        } else if (*property_request).ValueSize >= ulong_size {
            let flags = (*property_request).Value as *mut u32;
            if !flags.is_null() {
                *flags = KSPROPERTY_TYPE_GET | KSPROPERTY_TYPE_BASICSUPPORT;
            }
            (*property_request).ValueSize = ulong_size;
            return STATUS_SUCCESS;
        }
        return STATUS_BUFFER_TOO_SMALL;
    }

    DbgPrint(
        c"Leyline: jack_description_handler CALLED for ID %d, Pin %d\n".as_ptr(),
        prop_id,
        pin_id,
    );

    if prop_id == KSPROPERTY_JACK_DESCRIPTION {
        if (*property_request).ValueSize == 0 {
            (*property_request).ValueSize = core::mem::size_of::<KSJACK_DESCRIPTION>() as u32;
            return STATUS_BUFFER_OVERFLOW;
        }

        if (*property_request).ValueSize < core::mem::size_of::<KSJACK_DESCRIPTION>() as u32 {
            return STATUS_BUFFER_TOO_SMALL;
        }

        let jack_desc = (*property_request).Value as *mut KSJACK_DESCRIPTION;
        if !jack_desc.is_null() {
            (*jack_desc).ChannelMapping = 0x3; // KSAUDIO_SPEAKER_STEREO (FL | FR)
            (*jack_desc).Color = JACK_COLOR_BLACK;
            (*jack_desc).ConnectionType = 1; // eConnType3Point5mm
            (*jack_desc).GeoLocation = 1; // eGeoLocRear
            (*jack_desc).GenLocation = 0; // eGenLocPrimaryBox
            (*jack_desc).PortConnection = 0; // ePortConnJack
            (*jack_desc).IsConnected = 1; // TRUE
            DbgPrint(c"Leyline: jack_description_handler SUCCESS (IsConnected=1)\n".as_ptr());
        }
        return STATUS_SUCCESS;
    } else if prop_id == KSPROPERTY_JACK_DESCRIPTION2 {
        if (*property_request).ValueSize == 0 {
            (*property_request).ValueSize = core::mem::size_of::<KSJACK_DESCRIPTION2>() as u32;
            return STATUS_BUFFER_OVERFLOW;
        }

        if (*property_request).ValueSize < core::mem::size_of::<KSJACK_DESCRIPTION2>() as u32 {
            return STATUS_BUFFER_TOO_SMALL;
        }

        let jack_desc2 = (*property_request).Value as *mut KSJACK_DESCRIPTION2;
        if !jack_desc2.is_null() {
            (*jack_desc2).DeviceStateInfo = 0;
            (*jack_desc2).JackCapabilities = 0; // Capability: None
            DbgPrint(c"Leyline: jack_description_handler (JACK_DESCRIPTION2) SUCCESS\n".as_ptr());
        }
        return STATUS_SUCCESS;
    }

    STATUS_NOT_IMPLEMENTED
}

#[allow(clippy::missing_safety_doc)]
pub unsafe extern "C" fn mute_handler(property_request: PPCPROPERTY_REQUEST) -> NTSTATUS {
    if property_request.is_null() {
        return STATUS_INVALID_PARAMETER;
    }

    if ((*property_request).Verb & KSPROPERTY_TYPE_BASICSUPPORT) != 0 {
        let full_size = core::mem::size_of::<KSPROPERTY_DESCRIPTION>() as u32;
        let ulong_size = core::mem::size_of::<u32>() as u32;

        if (*property_request).ValueSize == 0 {
            (*property_request).ValueSize = full_size;
            return STATUS_BUFFER_OVERFLOW;
        }

        if (*property_request).ValueSize >= full_size {
            let desc = (*property_request).Value as *mut KSPROPERTY_DESCRIPTION;
            if !desc.is_null() {
                (*desc).AccessFlags =
                    KSPROPERTY_TYPE_GET | KSPROPERTY_TYPE_SET | KSPROPERTY_TYPE_BASICSUPPORT;
                (*desc).DescriptionSize = full_size;
                (*desc).PropTypeSet.Set = KSPROPTYPESETID_GENERAL_PROP;
                (*desc).PropTypeSet.Id = VT_BOOL;
                (*desc).PropTypeSet.Flags = 0;
                (*desc).MembersListCount = 0;
                (*desc).Reserved = 0;
            }
            (*property_request).ValueSize = full_size;
            return STATUS_SUCCESS;
        } else if (*property_request).ValueSize >= ulong_size {
            let flags = (*property_request).Value as *mut u32;
            if !flags.is_null() {
                *flags = KSPROPERTY_TYPE_GET | KSPROPERTY_TYPE_SET | KSPROPERTY_TYPE_BASICSUPPORT;
            }
            (*property_request).ValueSize = ulong_size;
            return STATUS_SUCCESS;
        }

        return STATUS_BUFFER_TOO_SMALL;
    }

    if (*property_request).ValueSize == 0 {
        (*property_request).ValueSize = core::mem::size_of::<i32>() as u32;
        return STATUS_BUFFER_OVERFLOW;
    }

    if (*property_request).ValueSize < core::mem::size_of::<i32>() as u32 {
        return STATUS_BUFFER_TOO_SMALL;
    }

    if (*property_request).Verb & KSPROPERTY_TYPE_GET != 0 {
        let value = (*property_request).Value as *mut i32;
        if !value.is_null() {
            *value = 0; // Unmuted by default
        }
    }

    STATUS_SUCCESS
}

#[allow(clippy::missing_safety_doc)]
pub unsafe extern "C" fn pin_category_handler(property_request: PPCPROPERTY_REQUEST) -> NTSTATUS {
    if property_request.is_null() {
        return STATUS_INVALID_PARAMETER;
    }

    if (*property_request).PropertyItem.is_null() {
        return STATUS_INVALID_PARAMETER;
    }

    let prop_id = (*(*property_request).PropertyItem).Id;
    DbgPrint(
        c"Leyline: pin_category_handler CALLED for ID %d\n".as_ptr(),
        prop_id,
    );
    STATUS_NOT_IMPLEMENTED
}

#[allow(clippy::missing_safety_doc)]
pub unsafe extern "C" fn pin_name_handler(property_request: PPCPROPERTY_REQUEST) -> NTSTATUS {
    if property_request.is_null() {
        return STATUS_INVALID_PARAMETER;
    }

    if (*property_request).PropertyItem.is_null() {
        return STATUS_INVALID_PARAMETER;
    }

    let prop_id = (*(*property_request).PropertyItem).Id;
    DbgPrint(
        c"Leyline: pin_name_handler CALLED for ID %d\n".as_ptr(),
        prop_id,
    );

    // This property returns a Unicode string for the pin name.
    // However, PortCls usually handles this by calling IMiniportPinName::GetPinName.
    // If we are here, it means PortCls is asking us to handle it manually or is passing it through.
    // For now, we'll return STATUS_NOT_IMPLEMENTED to let PortCls fall back to IPinName.
    // If the handshake still fails, we will implement full manual string allocation here.
    STATUS_NOT_IMPLEMENTED
}

#[link_section = ".rdata"]
pub static TOPO_FILTER_PROPERTIES: [PCPROPERTY_ITEM; 3] = [
    PCPROPERTY_ITEM {
        Set: &KSPROPSETID_GENERAL as *const GUID,
        Id: KSPROPERTY_GENERAL_COMPONENTID,
        Flags: KSPROPERTY_TYPE_GET | KSPROPERTY_TYPE_BASICSUPPORT,
        Handler: Some(component_id_handler),
    },
    PCPROPERTY_ITEM {
        Set: &KSPROPSETID_JACK as *const GUID,
        Id: KSPROPERTY_JACK_DESCRIPTION,
        Flags: KSPROPERTY_TYPE_GET | KSPROPERTY_TYPE_BASICSUPPORT,
        Handler: Some(jack_description_handler),
    },
    PCPROPERTY_ITEM {
        Set: &KSPROPSETID_JACK as *const GUID,
        Id: KSPROPERTY_JACK_DESCRIPTION2,
        Flags: KSPROPERTY_TYPE_GET | KSPROPERTY_TYPE_BASICSUPPORT,
        Handler: Some(jack_description_handler),
    },
];

#[allow(clippy::missing_safety_doc)]
pub unsafe extern "C" fn proposed_format_handler(
    property_request: PPCPROPERTY_REQUEST,
) -> NTSTATUS {
    if property_request.is_null() {
        return STATUS_INVALID_PARAMETER;
    }

    if (*property_request).PropertyItem.is_null() {
        return STATUS_INVALID_PARAMETER;
    }

    DbgPrint(c"Leyline: proposed_format_handler CALLED\n".as_ptr());

    let prop_id = (*(*property_request).PropertyItem).Id;

    // Handle Basic Support
    if ((*property_request).Verb & KSPROPERTY_TYPE_BASICSUPPORT) != 0 {
        let full_size = core::mem::size_of::<KSPROPERTY_DESCRIPTION>() as u32;
        let ulong_size = core::mem::size_of::<u32>() as u32;

        if (*property_request).ValueSize == 0 {
            (*property_request).ValueSize = full_size;
            return STATUS_BUFFER_OVERFLOW;
        }

        if (*property_request).ValueSize >= full_size {
            let desc = (*property_request).Value as *mut KSPROPERTY_DESCRIPTION;
            if !desc.is_null() {
                (*desc).AccessFlags =
                    KSPROPERTY_TYPE_GET | KSPROPERTY_TYPE_SET | KSPROPERTY_TYPE_BASICSUPPORT;
                (*desc).DescriptionSize = full_size;
                (*desc).PropTypeSet.Set = KSPROPTYPESETID_GENERAL_PROP;
                (*desc).PropTypeSet.Id = VT_I4;
                (*desc).PropTypeSet.Flags = 0;
                (*desc).MembersListCount = 0;
                (*desc).Reserved = 0;
            }
            (*property_request).ValueSize = full_size;
            return STATUS_SUCCESS;
        } else if (*property_request).ValueSize >= ulong_size {
            let flags = (*property_request).Value as *mut u32;
            if !flags.is_null() {
                *flags = KSPROPERTY_TYPE_GET | KSPROPERTY_TYPE_SET | KSPROPERTY_TYPE_BASICSUPPORT;
            }
            (*property_request).ValueSize = ulong_size;
            return STATUS_SUCCESS;
        }
        return STATUS_BUFFER_TOO_SMALL;
    }

    if ((*property_request).Verb & KSPROPERTY_TYPE_SET) != 0 {
        DbgPrint(c"Leyline: proposed_format_handler SET - OK\n".as_ptr());
        return STATUS_SUCCESS;
    }

    if ((*property_request).Verb & KSPROPERTY_TYPE_GET) != 0
        && (prop_id == KSPROPERTY_PIN_PROPOSEDATAFORMAT2
            || prop_id == KSPROPERTY_PIN_PROPOSEDATAFORMAT)
    {
        let format_size =
            core::mem::size_of::<crate::stream::KSDATAFORMAT_WAVEFORMATEXTENSIBLE>() as u32;
        if (*property_request).ValueSize == 0 {
            (*property_request).ValueSize = format_size;
            return STATUS_BUFFER_OVERFLOW;
        }
        if (*property_request).ValueSize < format_size {
            // If they provided less than extensible, but enough for EX, we could fall back.
            // But AEB usually asks for the full size.
            return STATUS_BUFFER_TOO_SMALL;
        }

        let result =
            (*property_request).Value as *mut crate::stream::KSDATAFORMAT_WAVEFORMATEXTENSIBLE;
        if !result.is_null() {
            (*result).DataFormat.FormatSize = format_size;
            (*result).DataFormat.MajorFormat = KSDATAFORMAT_TYPE_AUDIO;
            (*result).DataFormat.SubFormat = KSDATAFORMAT_SUBTYPE_PCM;
            (*result).DataFormat.Specifier = KSDATAFORMAT_SPECIFIER_WAVEFORMATEX; // Specifier stays the same for EXT

            (*result).WaveFormatExt.Format.wFormatTag = 0xFFFE; // WAVE_FORMAT_EXTENSIBLE
            (*result).WaveFormatExt.Format.nChannels = 2;
            (*result).WaveFormatExt.Format.nSamplesPerSec = 48000;
            (*result).WaveFormatExt.Format.wBitsPerSample = 16;
            (*result).WaveFormatExt.Format.nBlockAlign = 4;
            (*result).WaveFormatExt.Format.nAvgBytesPerSec = 48000 * 4;
            (*result).WaveFormatExt.Format.cbSize = 22; // Size of extensible part

            (*result).WaveFormatExt.Samples.wValidBitsPerSample = 16;
            (*result).WaveFormatExt.dwChannelMask = 0x3; // KSAUDIO_SPEAKER_STEREO
            (*result).WaveFormatExt.SubFormat = KSDATAFORMAT_SUBTYPE_PCM;

            (*result).DataFormat.SampleSize = 4;
            DbgPrint(
                c"Leyline: proposed_format_handler GET - Returning 48kHz Stereo (Extensible)\n"
                    .as_ptr(),
            );
        }
        return STATUS_SUCCESS;
    }

    STATUS_SUCCESS
}

#[allow(clippy::missing_safety_doc)]
pub unsafe extern "C" fn audio_effects_discovery_handler(
    property_request: PPCPROPERTY_REQUEST,
) -> NTSTATUS {
    if property_request.is_null() {
        return STATUS_INVALID_PARAMETER;
    }

    if (*property_request).PropertyItem.is_null() {
        return STATUS_INVALID_PARAMETER;
    }

    let prop_id = (*(*property_request).PropertyItem).Id;
    DbgPrint(
        c"Leyline: audio_effects_discovery_handler CALLED for ID %d\n".as_ptr(),
        prop_id,
    );

    // Handle Basic Support
    if ((*property_request).Verb & KSPROPERTY_TYPE_BASICSUPPORT) != 0 {
        if (*property_request).ValueSize < core::mem::size_of::<u32>() as u32 {
            (*property_request).ValueSize = core::mem::size_of::<u32>() as u32;
            return STATUS_BUFFER_OVERFLOW;
        }
        let flags = (*property_request).Value as *mut u32;
        if !flags.is_null() {
            *flags = KSPROPERTY_TYPE_GET | KSPROPERTY_TYPE_BASICSUPPORT;
        }
        (*property_request).ValueSize = core::mem::size_of::<u32>() as u32;
        return STATUS_SUCCESS;
    }

    // Start with NOT_IMPLEMENTED for GET until we have the effects list logic
    STATUS_NOT_IMPLEMENTED
}

#[allow(clippy::missing_safety_doc)]
pub unsafe extern "C" fn audio_module_handler(property_request: PPCPROPERTY_REQUEST) -> NTSTATUS {
    if property_request.is_null() {
        return STATUS_INVALID_PARAMETER;
    }

    if (*property_request).PropertyItem.is_null() {
        return STATUS_INVALID_PARAMETER;
    }

    let prop_id = (*(*property_request).PropertyItem).Id;
    DbgPrint(
        c"Leyline: audio_module_handler CALLED for ID %d\n".as_ptr(),
        prop_id,
    );

    // Handle Basic Support
    if ((*property_request).Verb & KSPROPERTY_TYPE_BASICSUPPORT) != 0 {
        if (*property_request).ValueSize < core::mem::size_of::<u32>() as u32 {
            (*property_request).ValueSize = core::mem::size_of::<u32>() as u32;
            return STATUS_BUFFER_OVERFLOW;
        }
        let flags = (*property_request).Value as *mut u32;
        if !flags.is_null() {
            *flags = if prop_id == KSPROPERTY_AUDIOMODULE_COMMAND {
                KSPROPERTY_TYPE_GET | KSPROPERTY_TYPE_SET | KSPROPERTY_TYPE_BASICSUPPORT
            } else {
                KSPROPERTY_TYPE_GET | KSPROPERTY_TYPE_BASICSUPPORT
            };
        }
        (*property_request).ValueSize = core::mem::size_of::<u32>() as u32;
        return STATUS_SUCCESS;
    }

    // Start with NOT_IMPLEMENTED
    STATUS_NOT_IMPLEMENTED
}

#[link_section = ".rdata"]
pub static WAVE_FILTER_PROPERTIES: [PCPROPERTY_ITEM; 9] = [
    PCPROPERTY_ITEM {
        Set: &KSPROPSETID_GENERAL as *const GUID,
        Id: KSPROPERTY_GENERAL_COMPONENTID,
        Flags: KSPROPERTY_TYPE_GET | KSPROPERTY_TYPE_BASICSUPPORT,
        Handler: Some(component_id_handler),
    },
    PCPROPERTY_ITEM {
        Set: &KSPROPSETID_PIN as *const GUID, // Using KSPROPSETID_PIN directly
        Id: KSPROPERTY_PIN_PROPOSEDATAFORMAT,
        Flags: KSPROPERTY_TYPE_GET | KSPROPERTY_TYPE_SET | KSPROPERTY_TYPE_BASICSUPPORT,
        Handler: Some(proposed_format_handler),
    },
    PCPROPERTY_ITEM {
        Set: &KSPROPSETID_PIN as *const GUID,
        Id: KSPROPERTY_PIN_PROPOSEDATAFORMAT2,
        Flags: KSPROPERTY_TYPE_GET | KSPROPERTY_TYPE_SET | KSPROPERTY_TYPE_BASICSUPPORT,
        Handler: Some(proposed_format_handler),
    },
    PCPROPERTY_ITEM {
        Set: &KSPROPSETID_JACK as *const GUID,
        Id: KSPROPERTY_JACK_DESCRIPTION,
        Flags: KSPROPERTY_TYPE_GET | KSPROPERTY_TYPE_BASICSUPPORT,
        Handler: Some(jack_description_handler),
    },
    PCPROPERTY_ITEM {
        Set: &KSPROPSETID_JACK as *const GUID,
        Id: KSPROPERTY_JACK_DESCRIPTION2,
        Flags: KSPROPERTY_TYPE_GET | KSPROPERTY_TYPE_BASICSUPPORT,
        Handler: Some(jack_description_handler),
    },
    PCPROPERTY_ITEM {
        Set: &KSPROPSETID_AUDIOEFFECTSDISCOVERY as *const GUID,
        Id: KSPROPERTY_AUDIOEFFECTSDISCOVERY_EFFECTSLIST,
        Flags: KSPROPERTY_TYPE_GET | KSPROPERTY_TYPE_BASICSUPPORT,
        Handler: Some(audio_effects_discovery_handler),
    },
    PCPROPERTY_ITEM {
        Set: &KSPROPSETID_AUDIOMODULE as *const GUID,
        Id: KSPROPERTY_AUDIOMODULE_DESCRIPTORS,
        Flags: KSPROPERTY_TYPE_GET | KSPROPERTY_TYPE_BASICSUPPORT,
        Handler: Some(audio_module_handler),
    },
    PCPROPERTY_ITEM {
        Set: &KSPROPSETID_AUDIOMODULE as *const GUID,
        Id: KSPROPERTY_AUDIOMODULE_COMMAND,
        Flags: KSPROPERTY_TYPE_GET | KSPROPERTY_TYPE_SET | KSPROPERTY_TYPE_BASICSUPPORT,
        Handler: Some(audio_module_handler),
    },
    PCPROPERTY_ITEM {
        Set: &KSPROPSETID_AUDIOMODULE as *const GUID,
        Id: KSPROPERTY_AUDIOMODULE_NOTIFICATION_DEVICE_ID,
        Flags: KSPROPERTY_TYPE_GET | KSPROPERTY_TYPE_BASICSUPPORT,
        Handler: Some(audio_module_handler),
    },
];

#[link_section = ".rdata"]
pub static WAVE_FILTER_AUTOMATION_TABLE: PCAUTOMATION_TABLE = PCAUTOMATION_TABLE {
    PropertyItemSize: core::mem::size_of::<PCPROPERTY_ITEM>() as u32,
    PropertyCount: 9,
    Properties: WAVE_FILTER_PROPERTIES.as_ptr(),
    MethodItemSize: 0,
    MethodCount: 0,
    Methods: core::ptr::null(),
    EventItemSize: 0,
    EventCount: 0,
    Events: core::ptr::null(),
    Reserved: 0,
};

#[link_section = ".rdata"]
pub static TOPO_FILTER_AUTOMATION_TABLE: PCAUTOMATION_TABLE = PCAUTOMATION_TABLE {
    PropertyItemSize: core::mem::size_of::<PCPROPERTY_ITEM>() as u32,
    PropertyCount: 3,
    Properties: TOPO_FILTER_PROPERTIES.as_ptr(),
    MethodItemSize: 0,
    MethodCount: 0,
    Methods: core::ptr::null(),
    EventItemSize: 0,
    EventCount: 0,
    Events: core::ptr::null(),
    Reserved: 0,
};

#[link_section = ".rdata"]
pub static COMPONENT_AUTOMATION_TABLE: PCAUTOMATION_TABLE = PCAUTOMATION_TABLE {
    PropertyItemSize: core::mem::size_of::<PCPROPERTY_ITEM>() as u32,
    PropertyCount: 1,
    Properties: GENERAL_PROPERTIES.as_ptr(),
    MethodItemSize: 0,
    MethodCount: 0,
    Methods: core::ptr::null(),
    EventItemSize: 0,
    EventCount: 0,
    Events: core::ptr::null(),
    Reserved: 0,
};

#[link_section = ".rdata"]
pub static MINIMAL_AUTOMATION_TABLE: PCAUTOMATION_TABLE = PCAUTOMATION_TABLE {
    PropertyItemSize: 0,
    PropertyCount: 0,
    Properties: core::ptr::null(),
    MethodItemSize: 0,
    MethodCount: 0,
    Methods: core::ptr::null(),
    EventItemSize: 0,
    EventCount: 0,
    Events: core::ptr::null(),
    Reserved: 0,
};

#[link_section = ".rdata"]
pub static PIN_PROPERTIES: [PCPROPERTY_ITEM; 4] = [
    PCPROPERTY_ITEM {
        Set: &KSPROPSETID_PIN as *const GUID, // Corrected casing in my head, verifying
        Id: KSPROPERTY_PIN_CATEGORY,
        Flags: KSPROPERTY_TYPE_GET | KSPROPERTY_TYPE_BASICSUPPORT,
        Handler: Some(pin_category_handler),
    },
    PCPROPERTY_ITEM {
        Set: &KSPROPSETID_PIN as *const GUID,
        Id: KSPROPERTY_PIN_NAME,
        Flags: KSPROPERTY_TYPE_GET | KSPROPERTY_TYPE_BASICSUPPORT,
        Handler: Some(pin_name_handler),
    },
    PCPROPERTY_ITEM {
        Set: &KSPROPSETID_JACK as *const GUID,
        Id: KSPROPERTY_JACK_DESCRIPTION,
        Flags: KSPROPERTY_TYPE_GET | KSPROPERTY_TYPE_BASICSUPPORT,
        Handler: Some(jack_description_handler),
    },
    PCPROPERTY_ITEM {
        Set: &KSPROPSETID_JACK as *const GUID,
        Id: KSPROPERTY_JACK_DESCRIPTION2,
        Flags: KSPROPERTY_TYPE_GET | KSPROPERTY_TYPE_BASICSUPPORT,
        Handler: Some(jack_description_handler),
    },
];

#[link_section = ".rdata"]
pub static PIN_AUTOMATION_TABLE: PCAUTOMATION_TABLE = PCAUTOMATION_TABLE {
    PropertyItemSize: core::mem::size_of::<PCPROPERTY_ITEM>() as u32,
    PropertyCount: 4,
    Properties: PIN_PROPERTIES.as_ptr(),
    MethodItemSize: 0,
    MethodCount: 0,
    Methods: core::ptr::null(),
    EventItemSize: 0,
    EventCount: 0,
    Events: core::ptr::null(),
    Reserved: 0,
};

// ===========================================================================
// Pin Descriptors
// ===========================================================================

// WAVE_RENDER_PINS
#[link_section = ".rdata"]
pub static WAVE_RENDER_PINS: [PCPIN_DESCRIPTOR; 2] = [
    PCPIN_DESCRIPTOR {
        MaxGlobalInstanceCount: 4,
        MaxFilterInstanceCount: 4,
        MinFilterInstanceCount: 1,
        AutomationTable: &PIN_AUTOMATION_TABLE,
        KsPinDescriptor: KSPIN_DESCRIPTOR {
            InterfacesCount: 1,
            Interfaces: KSINTERFACES.as_ptr() as *const core::ffi::c_void,
            MediumsCount: 0,
            Mediums: core::ptr::null(),
            DataRangesCount: 2,
            DataRanges: WAVE_DATARANGES.as_ptr() as *const *mut KSDATAFORMAT,
            DataFlow: KSPIN_DATAFLOW_IN as i32,
            Communication: KSPIN_COMMUNICATION_SINK as i32,
            Category: &KSCATEGORY_AUDIO_GUID as *const GUID,
            Name: core::ptr::null_mut(),
            Reserved: 0,
            Reserved2: 0,
        },
    },
    PCPIN_DESCRIPTOR {
        MaxGlobalInstanceCount: 1,
        MaxFilterInstanceCount: 1,
        MinFilterInstanceCount: 1,
        AutomationTable: &PIN_AUTOMATION_TABLE,
        KsPinDescriptor: KSPIN_DESCRIPTOR {
            // Bridge pins don't use the standard streaming interface.
            InterfacesCount: 0,
            Interfaces: core::ptr::null_mut(),
            MediumsCount: 0,
            Mediums: core::ptr::null_mut(),
            DataRangesCount: 1,
            DataRanges: BRIDGE_DATARANGES.as_ptr() as *const *mut KSDATAFORMAT,
            DataFlow: KSPIN_DATAFLOW_OUT as i32,
            Communication: KSPIN_COMMUNICATION_NONE as i32, // INTERNAL (NONE)
            Category: &KSCATEGORY_AUDIO_GUID as *const GUID, // MUST BE AUDIO CATEGORY
            Name: core::ptr::null_mut(),
            Reserved: 0,
            Reserved2: 0,
        },
    },
];

#[link_section = ".rdata"]
pub static WAVE_CAPTURE_PINS: [PCPIN_DESCRIPTOR; 2] = [
    PCPIN_DESCRIPTOR {
        MaxGlobalInstanceCount: 4,
        MaxFilterInstanceCount: 4,
        MinFilterInstanceCount: 1,
        AutomationTable: &PIN_AUTOMATION_TABLE,
        KsPinDescriptor: KSPIN_DESCRIPTOR {
            InterfacesCount: 1,
            Interfaces: KSINTERFACES.as_ptr() as *const core::ffi::c_void,
            MediumsCount: 0,
            Mediums: core::ptr::null_mut(),
            DataRangesCount: 2,
            DataRanges: WAVE_DATARANGES.as_ptr() as *const *mut KSDATAFORMAT,
            DataFlow: KSPIN_DATAFLOW_OUT as i32,
            Communication: KSPIN_COMMUNICATION_SINK as i32,
            Category: &KSCATEGORY_AUDIO_GUID as *const GUID,
            Name: core::ptr::null_mut(),
            Reserved: 0,
            Reserved2: 0,
        },
    },
    PCPIN_DESCRIPTOR {
        MaxGlobalInstanceCount: 1,
        MaxFilterInstanceCount: 1,
        MinFilterInstanceCount: 1,
        AutomationTable: &PIN_AUTOMATION_TABLE,
        KsPinDescriptor: KSPIN_DESCRIPTOR {
            // Bridge pins don't use the standard streaming interface.
            InterfacesCount: 0,
            Interfaces: core::ptr::null_mut(),
            MediumsCount: 0,
            Mediums: core::ptr::null_mut(),
            DataRangesCount: 1,
            DataRanges: BRIDGE_DATARANGES.as_ptr() as *const *mut KSDATAFORMAT,
            DataFlow: KSPIN_DATAFLOW_IN as i32,
            Communication: KSPIN_COMMUNICATION_NONE as i32, // INTERNAL (NONE)
            Category: &KSCATEGORY_AUDIO_GUID as *const GUID, // AEB requires this
            Name: core::ptr::null_mut(),
            Reserved: 0,
            Reserved2: 0,
        },
    },
];

// TOPO_RENDER_PINS
#[link_section = ".rdata"]
pub static TOPO_RENDER_PINS: [PCPIN_DESCRIPTOR; 2] = [
    PCPIN_DESCRIPTOR {
        MaxGlobalInstanceCount: 1,
        MaxFilterInstanceCount: 1,
        MinFilterInstanceCount: 1,
        AutomationTable: &PIN_AUTOMATION_TABLE,
        KsPinDescriptor: KSPIN_DESCRIPTOR {
            InterfacesCount: 0,
            Interfaces: core::ptr::null_mut(),
            MediumsCount: 0,
            Mediums: core::ptr::null_mut(),
            DataRangesCount: 1,
            DataRanges: BRIDGE_DATARANGES.as_ptr() as *const *mut KSDATAFORMAT,
            DataFlow: KSPIN_DATAFLOW_IN as i32,
            Communication: KSPIN_COMMUNICATION_NONE as i32,
            Category: &KSCATEGORY_AUDIO_GUID as *const GUID, // Input from Wave
            Name: core::ptr::null_mut(),
            Reserved: 0,
            Reserved2: 0,
        },
    },
    PCPIN_DESCRIPTOR {
        MaxGlobalInstanceCount: 1,
        MaxFilterInstanceCount: 1,
        MinFilterInstanceCount: 1,
        AutomationTable: &PIN_AUTOMATION_TABLE,
        KsPinDescriptor: KSPIN_DESCRIPTOR {
            InterfacesCount: 0,
            Interfaces: core::ptr::null_mut(),
            MediumsCount: 0,
            Mediums: core::ptr::null_mut(),
            DataRangesCount: 1,
            DataRanges: BRIDGE_DATARANGES.as_ptr() as *const *mut KSDATAFORMAT,
            DataFlow: KSPIN_DATAFLOW_OUT as i32,
            Communication: KSPIN_COMMUNICATION_NONE as i32,
            Category: &KSNODETYPE_SPEAKER as *const GUID, // Output to Speaker
            Name: core::ptr::null_mut(),
            Reserved: 0,
            Reserved2: 0,
        },
    },
];

#[link_section = ".rdata"]
pub static TOPO_CAPTURE_PINS: [PCPIN_DESCRIPTOR; 2] = [
    PCPIN_DESCRIPTOR {
        MaxGlobalInstanceCount: 1,
        MaxFilterInstanceCount: 1,
        MinFilterInstanceCount: 1,
        AutomationTable: &PIN_AUTOMATION_TABLE,
        KsPinDescriptor: KSPIN_DESCRIPTOR {
            InterfacesCount: 0,
            Interfaces: core::ptr::null_mut(),
            MediumsCount: 0,
            Mediums: core::ptr::null_mut(),
            DataRangesCount: 1,
            DataRanges: BRIDGE_DATARANGES.as_ptr() as *const *mut KSDATAFORMAT,
            DataFlow: KSPIN_DATAFLOW_IN as i32,
            Communication: KSPIN_COMMUNICATION_NONE as i32,
            Category: &KSNODETYPE_MICROPHONE as *const GUID, // Input from Mic
            Name: core::ptr::null_mut(),
            Reserved: 0,
            Reserved2: 0,
        },
    },
    PCPIN_DESCRIPTOR {
        MaxGlobalInstanceCount: 1,
        MaxFilterInstanceCount: 1,
        MinFilterInstanceCount: 1,
        AutomationTable: &PIN_AUTOMATION_TABLE,
        KsPinDescriptor: KSPIN_DESCRIPTOR {
            InterfacesCount: 0,
            Interfaces: core::ptr::null_mut(),
            MediumsCount: 0,
            Mediums: core::ptr::null_mut(),
            DataRangesCount: 1,
            DataRanges: BRIDGE_DATARANGES.as_ptr() as *const *mut KSDATAFORMAT,
            DataFlow: KSPIN_DATAFLOW_OUT as i32,
            Communication: KSPIN_COMMUNICATION_NONE as i32,
            Category: &KSCATEGORY_AUDIO_GUID as *const GUID, // Output to Wave
            Name: core::ptr::null_mut(),
            Reserved: 0,
            Reserved2: 0,
        },
    },
];

// ===========================================================================
// Connections & Categories
// ===========================================================================

// ===========================================================================
// Property Handlers (Volume)
// ===========================================================================

#[allow(clippy::missing_safety_doc)]
pub unsafe extern "C" fn volume_handler(property_request: PPCPROPERTY_REQUEST) -> NTSTATUS {
    if property_request.is_null() {
        return STATUS_INVALID_PARAMETER;
    }

    #[repr(C)]
    struct VOL_BASIC {
        desc: KSPROPERTY_DESCRIPTION,
        hdr: KSPROPERTY_MEMBERSHEADER,
        stepping: KSPROPERTY_STEPPING_LONG,
    }

    if ((*property_request).Verb & KSPROPERTY_TYPE_BASICSUPPORT) != 0 {
        let full_size = core::mem::size_of::<VOL_BASIC>() as u32;
        let ulong_size = core::mem::size_of::<u32>() as u32;

        if (*property_request).ValueSize == 0 {
            (*property_request).ValueSize = full_size;
            return STATUS_BUFFER_OVERFLOW;
        }

        if (*property_request).ValueSize >= full_size {
            let vol_basic = (*property_request).Value as *mut VOL_BASIC;
            if !vol_basic.is_null() {
                (*vol_basic).desc.AccessFlags =
                    KSPROPERTY_TYPE_GET | KSPROPERTY_TYPE_SET | KSPROPERTY_TYPE_BASICSUPPORT;
                (*vol_basic).desc.DescriptionSize = full_size;
                (*vol_basic).desc.PropTypeSet.Set = KSPROPTYPESETID_GENERAL_PROP;
                (*vol_basic).desc.PropTypeSet.Id = VT_I4;
                (*vol_basic).desc.PropTypeSet.Flags = 0;
                (*vol_basic).desc.MembersListCount = 1;
                (*vol_basic).desc.Reserved = 0;

                (*vol_basic).hdr.MembersFlags = KSPROPERTY_MEMBER_STEPPEDRANGES;
                (*vol_basic).hdr.MembersSize =
                    core::mem::size_of::<KSPROPERTY_STEPPING_LONG>() as u32;
                (*vol_basic).hdr.MembersCount = 1;
                (*vol_basic).hdr.Flags = 0;

                (*vol_basic).stepping.SignedMaximum = 0x00000000; // 0 dB
                (*vol_basic).stepping.SignedMinimum = -96 * 0x10000; // -96 dB
                (*vol_basic).stepping.SteppingDelta = 0x10000; // 1 dB
                (*vol_basic).stepping.Reserved = 0;
            }
            (*property_request).ValueSize = full_size;
            return STATUS_SUCCESS;
        } else if (*property_request).ValueSize >= ulong_size {
            let flags = (*property_request).Value as *mut u32;
            if !flags.is_null() {
                *flags = KSPROPERTY_TYPE_GET | KSPROPERTY_TYPE_SET | KSPROPERTY_TYPE_BASICSUPPORT;
            }
            (*property_request).ValueSize = ulong_size;
            return STATUS_SUCCESS;
        }

        return STATUS_BUFFER_TOO_SMALL;
    }

    if (*property_request).ValueSize == 0 {
        (*property_request).ValueSize = core::mem::size_of::<i32>() as u32;
        return STATUS_BUFFER_OVERFLOW;
    }

    if (*property_request).ValueSize < core::mem::size_of::<i32>() as u32 {
        return STATUS_BUFFER_TOO_SMALL;
    }

    if (*property_request).Verb & KSPROPERTY_TYPE_GET != 0 {
        let value = (*property_request).Value as *mut i32;
        if !value.is_null() {
            *value = 0;
        }
    }

    STATUS_SUCCESS
}

#[link_section = ".rdata"]
pub static VOLUME_PROPERTIES: [PCPROPERTY_ITEM; 1] = [PCPROPERTY_ITEM {
    Set: &KSPROPSETID_AUDIO as *const GUID,
    Id: KSPROPERTY_AUDIO_VOLUMELEVEL,
    Flags: KSPROPERTY_TYPE_GET | KSPROPERTY_TYPE_SET | KSPROPERTY_TYPE_BASICSUPPORT,
    Handler: Some(volume_handler),
}];

#[link_section = ".rdata"]
pub static VOLUME_AUTOMATION_TABLE: PCAUTOMATION_TABLE = PCAUTOMATION_TABLE {
    PropertyItemSize: core::mem::size_of::<PCPROPERTY_ITEM>() as u32,
    PropertyCount: 1,
    Properties: VOLUME_PROPERTIES.as_ptr(),
    MethodItemSize: 0,
    MethodCount: 0,
    Methods: core::ptr::null(),
    EventItemSize: 0,
    EventCount: 0,
    Events: core::ptr::null(),
    Reserved: 0,
};

#[link_section = ".rdata"]
pub static MUTE_PROPERTIES: [PCPROPERTY_ITEM; 1] = [PCPROPERTY_ITEM {
    Set: &KSPROPSETID_AUDIO as *const GUID,
    Id: KSPROPERTY_AUDIO_MUTE,
    Flags: KSPROPERTY_TYPE_GET | KSPROPERTY_TYPE_SET | KSPROPERTY_TYPE_BASICSUPPORT,
    Handler: Some(mute_handler),
}];

#[link_section = ".rdata"]
pub static MUTE_AUTOMATION_TABLE: PCAUTOMATION_TABLE = PCAUTOMATION_TABLE {
    PropertyItemSize: core::mem::size_of::<PCPROPERTY_ITEM>() as u32,
    PropertyCount: 1,
    Properties: MUTE_PROPERTIES.as_ptr(),
    MethodItemSize: 0,
    MethodCount: 0,
    Methods: core::ptr::null(),
    EventItemSize: 0,
    EventCount: 0,
    Events: core::ptr::null(),
    Reserved: 0,
};

#[link_section = ".rdata"]
pub static TOPO_NODES: [crate::stream::PCNODE_DESCRIPTOR; 2] = [
    crate::stream::PCNODE_DESCRIPTOR {
        Flags: 0,
        AutomationTable: &VOLUME_AUTOMATION_TABLE,
        Type: &KSNODETYPE_VOLUME,
        Name: &KSAUDFNAME_MASTER_VOLUME,
    },
    crate::stream::PCNODE_DESCRIPTOR {
        Flags: 0,
        AutomationTable: &MUTE_AUTOMATION_TABLE,
        Type: &KSNODETYPE_MUTE,
        Name: &KSAUDFNAME_MASTER_MUTE,
    },
];

// ===========================================================================
// Connections & Categories
// ===========================================================================

#[link_section = ".rdata"]
pub static WAVE_CONNECTIONS: [PCCONNECTION; 1] = [PCCONNECTION {
    FromNode: PCFILTER_NODE,
    FromNodePin: KSPIN_WAVE_SINK,
    ToNode: PCFILTER_NODE,
    ToNodePin: KSPIN_WAVE_BRIDGE,
}];

#[link_section = ".rdata"]
pub static WAVE_CAPTURE_CONNECTIONS: [PCCONNECTION; 1] = [PCCONNECTION {
    FromNode: PCFILTER_NODE,
    FromNodePin: KSPIN_WAVE_BRIDGE,
    ToNode: PCFILTER_NODE,
    ToNodePin: KSPIN_WAVE_SINK,
}];

// KSJACK_DESCRIPTION2 struct definition removed (duplicate)

#[link_section = ".rdata"]
pub static TOPO_CONNECTIONS: [PCCONNECTION; 3] = [
    PCCONNECTION {
        FromNode: PCFILTER_NODE,
        FromNodePin: KSPIN_TOPO_BRIDGE,
        ToNode: 0, // VOLUME
        ToNodePin: 1,
    },
    PCCONNECTION {
        FromNode: 0, // VOLUME
        FromNodePin: 0,
        ToNode: 1, // MUTE
        ToNodePin: 1,
    },
    PCCONNECTION {
        FromNode: 1, // MUTE
        FromNodePin: 0,
        ToNode: PCFILTER_NODE,
        ToNodePin: KSPIN_TOPO_LINEOUT,
    },
];

#[link_section = ".rdata"]
pub static TOPO_CAPTURE_CONNECTIONS: [PCCONNECTION; 1] = [PCCONNECTION {
    FromNode: PCFILTER_NODE,
    FromNodePin: 0, // MICROPHONE pin
    ToNode: PCFILTER_NODE,
    ToNodePin: 1, // BRIDGE pin
}];

#[link_section = ".rdata"]
pub static TOPO_FILTER_CATEGORIES: [GUID; 2] = [KSCATEGORY_AUDIO_GUID, KSCATEGORY_TOPOLOGY_GUID];
#[link_section = ".rdata"]
pub static WAVE_RENDER_CATEGORIES: [GUID; 3] = [
    KSCATEGORY_AUDIO_GUID,
    KSCATEGORY_RENDER_GUID,
    KSCATEGORY_REALTIME_GUID,
];
#[link_section = ".rdata"]
pub static WAVE_CAPTURE_CATEGORIES: [GUID; 3] = [
    KSCATEGORY_AUDIO_GUID,
    KSCATEGORY_CAPTURE_GUID,
    KSCATEGORY_REALTIME_GUID,
];

// ===========================================================================
// Filter Descriptors
// ===========================================================================

#[link_section = ".rdata"]
pub static WAVE_RENDER_FILTER_DESCRIPTOR: PCFILTER_DESCRIPTOR = PCFILTER_DESCRIPTOR {
    Version: 0,
    AutomationTable: &WAVE_FILTER_AUTOMATION_TABLE,
    PinSize: core::mem::size_of::<PCPIN_DESCRIPTOR>() as u32,
    PinCount: 2,
    Pins: WAVE_RENDER_PINS.as_ptr(),
    NodeSize: 0,
    NodeCount: 0,
    Nodes: core::ptr::null(),
    // CRITICAL: Wire in the pin-to-pin connection (Sink -> Bridge).
    // Without this, PortCls sees two disconnected pins and AEB aborts path traversal.
    ConnectionCount: 1,
    Connections: WAVE_CONNECTIONS.as_ptr(),
    CategoryCount: 3,
    Categories: WAVE_RENDER_CATEGORIES.as_ptr(),
};

#[link_section = ".rdata"]
pub static WAVE_CAPTURE_FILTER_DESCRIPTOR: PCFILTER_DESCRIPTOR = PCFILTER_DESCRIPTOR {
    Version: 0,
    AutomationTable: &WAVE_FILTER_AUTOMATION_TABLE,
    PinSize: core::mem::size_of::<PCPIN_DESCRIPTOR>() as u32,
    PinCount: 2,
    Pins: WAVE_CAPTURE_PINS.as_ptr(),
    NodeSize: 0,
    NodeCount: 0,
    Nodes: core::ptr::null(),
    // CRITICAL: Wire in the pin-to-pin connection (Bridge -> Sink).
    // Capture data flows: Bridge (from topo) -> Sink (to app client).
    ConnectionCount: 1,
    Connections: WAVE_CAPTURE_CONNECTIONS.as_ptr(),
    CategoryCount: 3,
    Categories: WAVE_CAPTURE_CATEGORIES.as_ptr(),
};

#[link_section = ".rdata"]
pub static TOPO_RENDER_FILTER_DESCRIPTOR: PCFILTER_DESCRIPTOR = PCFILTER_DESCRIPTOR {
    Version: 0,
    AutomationTable: &TOPO_FILTER_AUTOMATION_TABLE,
    PinSize: core::mem::size_of::<PCPIN_DESCRIPTOR>() as u32,
    PinCount: 2,
    Pins: TOPO_RENDER_PINS.as_ptr(),
    NodeSize: core::mem::size_of::<crate::stream::PCNODE_DESCRIPTOR>() as u32,
    NodeCount: 2,
    Nodes: TOPO_NODES.as_ptr(),
    ConnectionCount: 3,
    Connections: TOPO_CONNECTIONS.as_ptr(),
    CategoryCount: 2,
    Categories: TOPO_FILTER_CATEGORIES.as_ptr(),
};

#[link_section = ".rdata"]
pub static TOPO_CAPTURE_FILTER_DESCRIPTOR: PCFILTER_DESCRIPTOR = PCFILTER_DESCRIPTOR {
    Version: 0,
    AutomationTable: &TOPO_FILTER_AUTOMATION_TABLE,
    PinSize: core::mem::size_of::<PCPIN_DESCRIPTOR>() as u32,
    PinCount: 2,
    Pins: TOPO_CAPTURE_PINS.as_ptr(),
    NodeSize: 0,
    NodeCount: 0,
    Nodes: core::ptr::null(),
    ConnectionCount: 1,
    Connections: TOPO_CAPTURE_CONNECTIONS.as_ptr(),
    CategoryCount: 2,
    Categories: TOPO_FILTER_CATEGORIES.as_ptr(),
};
