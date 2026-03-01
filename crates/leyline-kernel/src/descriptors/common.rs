// Copyright (c) 2026 Randall Rosas (Slategray).
// All rights reserved.

// ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
// COMMON KERNEL STREAMING TYPES & HANDLERS
// ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

use wdk_sys::ntddk::*;
use wdk_sys::*;

use crate::constants::*;
use crate::stream::{
    KSDATAFORMAT, KSDATARANGE, KSDATARANGE_AUDIO, PCAUTOMATION_TABLE, PCPROPERTY_ITEM,
    PPCPROPERTY_REQUEST,
};

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

pub const KSPROPERTY_MEMBER_STEPPEDRANGES: u32 = 2;
pub const VT_I4: u32 = 3;
pub const VT_BOOL: u32 = 11;

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

/// Provide driver identity information.
pub unsafe extern "C" fn component_id_handler(property_request: PPCPROPERTY_REQUEST) -> NTSTATUS {
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

    if (*property_request).ValueSize == 0 {
        (*property_request).ValueSize = core::mem::size_of::<KSCOMPONENTID>() as u32;
        return STATUS_BUFFER_OVERFLOW;
    }

    if (*property_request).ValueSize < core::mem::size_of::<KSCOMPONENTID>() as u32 {
        return STATUS_BUFFER_TOO_SMALL;
    }

    let component_id = (*property_request).Value as *mut KSCOMPONENTID;
    if !component_id.is_null() {
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

/// Provide jack connection state and description.
pub unsafe extern "C" fn jack_description_handler(
    property_request: PPCPROPERTY_REQUEST,
) -> NTSTATUS {
    if property_request.is_null() {
        return STATUS_INVALID_PARAMETER;
    }

    if (*property_request).PropertyItem.is_null() {
        return STATUS_INVALID_PARAMETER;
    }

    let prop_id = (*(*property_request).PropertyItem).Id;
    let mut pin_id = !0u32;
    if (*property_request).InstanceSize >= core::mem::size_of::<u32>() as u32 {
        pin_id = *((*property_request).Instance as *const u32);
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
            (*jack_desc).ChannelMapping = 0x3; // KSAUDIO_SPEAKER_STEREO
            (*jack_desc).Color = JACK_COLOR_BLACK;
            (*jack_desc).ConnectionType = 1; // eConnType3Point5mm
            (*jack_desc).GeoLocation = 1; // eGeoLocRear
            (*jack_desc).GenLocation = 0; // eGenLocPrimaryBox
            (*jack_desc).PortConnection = 0; // ePortConnJack
            (*jack_desc).IsConnected = 1;
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
            (*jack_desc2).JackCapabilities = 0;
        }
        return STATUS_SUCCESS;
    }

    STATUS_NOT_IMPLEMENTED
}

/// Handle mute control requests.
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
            *value = 0;
        }
    }

    STATUS_SUCCESS
}

/// Handle volume control requests.
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

                (*vol_basic).stepping.SignedMaximum = 0x00000000;
                (*vol_basic).stepping.SignedMinimum = -96 * 0x10000;
                (*vol_basic).stepping.SteppingDelta = 0x10000;
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
pub static PIN_PROPERTIES: [PCPROPERTY_ITEM; 4] = [
    PCPROPERTY_ITEM {
        Set: &KSPROPSETID_PIN as *const GUID,
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

/// Handle pin category information requests.
pub unsafe extern "C" fn pin_category_handler(property_request: PPCPROPERTY_REQUEST) -> NTSTATUS {
    if property_request.is_null() {
        return STATUS_INVALID_PARAMETER;
    }
    STATUS_NOT_IMPLEMENTED
}

/// Handle pin name requests.
pub unsafe extern "C" fn pin_name_handler(property_request: PPCPROPERTY_REQUEST) -> NTSTATUS {
    if property_request.is_null() {
        return STATUS_INVALID_PARAMETER;
    }
    STATUS_NOT_IMPLEMENTED
}

/// Handle proposed audio format requests.
pub unsafe extern "C" fn proposed_format_handler(
    property_request: PPCPROPERTY_REQUEST,
) -> NTSTATUS {
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
        return STATUS_SUCCESS;
    }

    if ((*property_request).Verb & KSPROPERTY_TYPE_GET) != 0 {
        let format_size =
            core::mem::size_of::<crate::stream::KSDATAFORMAT_WAVEFORMATEXTENSIBLE>() as u32;
        if (*property_request).ValueSize == 0 {
            (*property_request).ValueSize = format_size;
            return STATUS_BUFFER_OVERFLOW;
        }
        if (*property_request).ValueSize < format_size {
            return STATUS_BUFFER_TOO_SMALL;
        }

        let result =
            (*property_request).Value as *mut crate::stream::KSDATAFORMAT_WAVEFORMATEXTENSIBLE;
        if !result.is_null() {
            (*result).DataFormat.FormatSize = format_size;
            (*result).DataFormat.MajorFormat = KSDATAFORMAT_TYPE_AUDIO;
            (*result).DataFormat.SubFormat = KSDATAFORMAT_SUBTYPE_PCM;
            (*result).DataFormat.Specifier = KSDATAFORMAT_SPECIFIER_WAVEFORMATEX;

            (*result).WaveFormatExt.Format.wFormatTag = 0xFFFE;
            (*result).WaveFormatExt.Format.nChannels = 2;
            (*result).WaveFormatExt.Format.nSamplesPerSec = 48000;
            (*result).WaveFormatExt.Format.wBitsPerSample = 16;
            (*result).WaveFormatExt.Format.nBlockAlign = 4;
            (*result).WaveFormatExt.Format.nAvgBytesPerSec = 48000 * 4;
            (*result).WaveFormatExt.Format.cbSize = 22;

            (*result).WaveFormatExt.Samples.wValidBitsPerSample = 16;
            (*result).WaveFormatExt.dwChannelMask = 0x3;
            (*result).WaveFormatExt.SubFormat = KSDATAFORMAT_SUBTYPE_PCM;

            (*result).DataFormat.SampleSize = 4;
        }
        return STATUS_SUCCESS;
    }

    STATUS_SUCCESS
}

/// Handle audio effects discovery requests.
pub unsafe extern "C" fn audio_effects_discovery_handler(
    property_request: PPCPROPERTY_REQUEST,
) -> NTSTATUS {
    if property_request.is_null() {
        return STATUS_INVALID_PARAMETER;
    }

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

    STATUS_NOT_IMPLEMENTED
}

/// Handle audio module configuration requests.
pub unsafe extern "C" fn audio_module_handler(property_request: PPCPROPERTY_REQUEST) -> NTSTATUS {
    if property_request.is_null() {
        return STATUS_INVALID_PARAMETER;
    }

    if (*property_request).PropertyItem.is_null() {
        return STATUS_INVALID_PARAMETER;
    }

    let prop_id = (*(*property_request).PropertyItem).Id;

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

    STATUS_NOT_IMPLEMENTED
}
