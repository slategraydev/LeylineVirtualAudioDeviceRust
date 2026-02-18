// Copyright (c) 2026 Randall Rosas (Slategray).
// All rights reserved.
//
// This source code is provided for educational and review purposes.
// Redistribution and use in binary form without express permission is prohibited.
// See LICENSE file in the project root for full terms.

// Second, external crates.
use wdk_sys::ntddk::*;
use wdk_sys::{
    GUID, NTSTATUS, STATUS_BUFFER_OVERFLOW, STATUS_BUFFER_TOO_SMALL, STATUS_INVALID_PARAMETER,
    STATUS_NOT_IMPLEMENTED, STATUS_SUCCESS,
};

// Local KSCOMPONENTID definition matching the Windows SDK layout exactly.
// Field order MUST match: Manufacturer, Product, Component, Name, Version, Revision.
// Total size = 4 GUIDs (64 bytes) + 2 u32s (8 bytes) = 72 bytes.
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
unsafe impl Sync for KSCOMPONENTID {}
unsafe impl Send for KSCOMPONENTID {}

#[repr(C)]
#[allow(non_snake_case)]
pub struct KSJACK_DESCRIPTION {
    pub ChannelMapping: u32,
    pub Color: u32,
    pub ConnectionType: u32,
    pub GeoLocation: u32,
    pub GenLocation: u32,
    pub PortConnection: u32,
    pub IsConnected: i32, // BOOL
}

#[repr(C)]
#[allow(non_snake_case)]
pub struct KSJACK_DESCRIPTION2 {
    pub DeviceStateInfo: u32,
    pub JackCapabilities: u32,
}

// Colors
pub const JACK_COLOR_BLACK: u32 = 0x00000000;
// Connection Type
#[allow(non_upper_case_globals)]
pub const eConnType3Point5mm: u32 = 1;
// GeoLocation
#[allow(non_upper_case_globals)]
pub const eGeoLocRear: u32 = 1;
// GenLocation
#[allow(non_upper_case_globals)]
pub const eGenLocPrimaryBox: u32 = 0;
// PortConnection
#[allow(non_upper_case_globals)]
pub const ePortConnJack: u32 = 0;

// Then current crate.
use crate::constants::*;
use crate::stream::{
    KSDATAFORMAT, KSDATARANGE, KSDATARANGE_AUDIO, KSPIN_DESCRIPTOR, PCAUTOMATION_TABLE,
    PCCONNECTION, PCFILTER_DESCRIPTOR, PCPIN_DESCRIPTOR, PCPROPERTY_ITEM, PPCPROPERTY_REQUEST,
};

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

#[link_section = ".rdata"]
pub static KSINTERFACES: [KSIDENTIFIER; 1] = [KSIDENTIFIER {
    Set: KSINTERFACESETID_STANDARD,
    Id: 0, // KSINTERFACE_STANDARD_STREAMING
    Flags: 0,
}];

#[repr(transparent)]
pub struct SyncPtr<T>(pub *const T);
unsafe impl<T> Sync for SyncPtr<T> {}

// ============================================================================
// Data Ranges
// ============================================================================

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
    MinimumBitsPerSample: 16,
    MaximumBitsPerSample: 32,
    MinimumSampleFrequency: 44100,
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
    MinimumBitsPerSample: 32,
    MaximumBitsPerSample: 32,
    MinimumSampleFrequency: 44100,
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
    Set: &KSPROPSETID_General as *const GUID,
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

    // Handle Basic Support
    if ((*property_request).Verb & KSPROPERTY_TYPE_BASICSUPPORT) != 0 {
        if (*property_request).ValueSize < core::mem::size_of::<u32>() as u32 {
            (*property_request).ValueSize = core::mem::size_of::<u32>() as u32;
            return STATUS_BUFFER_OVERFLOW;
        }
        let flags = (*property_request).Value as *mut u32;
        if !flags.is_null() {
            // Jack Description is GET only.
            *flags = KSPROPERTY_TYPE_GET | KSPROPERTY_TYPE_BASICSUPPORT;
        }
        (*property_request).ValueSize = core::mem::size_of::<u32>() as u32;
        return STATUS_SUCCESS;
    }

    DbgPrint(
        c"Leyline: jack_description_handler CALLED for ID %d\n".as_ptr(),
        prop_id,
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
            (*jack_desc).ChannelMapping = 0;
            (*jack_desc).Color = JACK_COLOR_BLACK;
            (*jack_desc).ConnectionType = eConnType3Point5mm;
            (*jack_desc).GeoLocation = eGeoLocRear;
            (*jack_desc).GenLocation = eGenLocPrimaryBox;
            (*jack_desc).PortConnection = ePortConnJack;
            (*jack_desc).IsConnected = 1; // Always Connected
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
            (*jack_desc2).JackCapabilities = 1; // Capability: Presence Detect
            DbgPrint(c"Leyline: jack_description_handler (JACK_DESCRIPTION2) SUCCESS\n".as_ptr());
        }
        return STATUS_SUCCESS;
    }

    STATUS_NOT_IMPLEMENTED
}

#[allow(clippy::missing_safety_doc)]
pub unsafe extern "C" fn pin_category_handler(property_request: PPCPROPERTY_REQUEST) -> NTSTATUS {
    if property_request.is_null() {
        return STATUS_INVALID_PARAMETER;
    }

    DbgPrint(c"Leyline: pin_category_handler CALLED\n".as_ptr());
    STATUS_NOT_IMPLEMENTED
}

#[allow(clippy::missing_safety_doc)]
pub unsafe extern "C" fn pin_name_handler(property_request: PPCPROPERTY_REQUEST) -> NTSTATUS {
    if property_request.is_null() {
        return STATUS_INVALID_PARAMETER;
    }

    DbgPrint(c"Leyline: pin_name_handler CALLED\n".as_ptr());

    // This property returns a Unicode string for the pin name.
    // However, PortCls usually handles this by calling IMiniportPinName::GetPinName.
    // If we are here, it means PortCls is asking us to handle it manually or is passing it through.
    // For now, we'll return STATUS_NOT_IMPLEMENTED to let PortCls fall back to IPinName.
    // If the handshake still fails, we will implement full manual string allocation here.
    STATUS_NOT_IMPLEMENTED
}

#[link_section = ".rdata"]
pub static TOPO_PIN_PROPERTIES: [PCPROPERTY_ITEM; 2] = [
    PCPROPERTY_ITEM {
        Set: &KSPROPSETID_Jack as *const GUID,
        Id: KSPROPERTY_JACK_DESCRIPTION,
        Flags: KSPROPERTY_TYPE_GET | KSPROPERTY_TYPE_BASICSUPPORT,
        Handler: Some(jack_description_handler),
    },
    PCPROPERTY_ITEM {
        Set: &KSPROPSETID_Jack as *const GUID,
        Id: KSPROPERTY_JACK_DESCRIPTION2,
        Flags: KSPROPERTY_TYPE_GET | KSPROPERTY_TYPE_BASICSUPPORT,
        Handler: Some(jack_description_handler),
    },
];

#[link_section = ".rdata"]
pub static TOPO_PIN_AUTOMATION_TABLE: PCAUTOMATION_TABLE = PCAUTOMATION_TABLE {
    PropertyItemSize: core::mem::size_of::<PCPROPERTY_ITEM>() as u32,
    PropertyCount: 2,
    Properties: TOPO_PIN_PROPERTIES.as_ptr(),
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
pub static PIN_PROPERTIES: [PCPROPERTY_ITEM; 0] = [];

#[link_section = ".rdata"]
pub static PIN_AUTOMATION_TABLE: PCAUTOMATION_TABLE = PCAUTOMATION_TABLE {
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

// ============================================================================
// Pin Descriptors
// ============================================================================

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
            Name: core::ptr::null(),
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
            InterfacesCount: 1,
            Interfaces: KSINTERFACES.as_ptr() as *const core::ffi::c_void,
            MediumsCount: 0,
            Mediums: core::ptr::null(),
            DataRangesCount: 1,
            DataRanges: BRIDGE_DATARANGES.as_ptr() as *const *mut KSDATAFORMAT,
            DataFlow: KSPIN_DATAFLOW_OUT as i32,
            Communication: KSPIN_COMMUNICATION_NONE as i32, // INTERNAL (NONE)
            Category: &KSCATEGORY_AUDIO_GUID as *const GUID, // Or NONE?
            Name: core::ptr::null(),
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
            Mediums: core::ptr::null(),
            DataRangesCount: 2,
            DataRanges: WAVE_DATARANGES.as_ptr() as *const *mut KSDATAFORMAT,
            DataFlow: KSPIN_DATAFLOW_OUT as i32,
            Communication: KSPIN_COMMUNICATION_SINK as i32,
            Category: &KSCATEGORY_AUDIO_GUID as *const GUID,
            Name: core::ptr::null(),
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
            InterfacesCount: 1,
            Interfaces: KSINTERFACES.as_ptr() as *const core::ffi::c_void,
            MediumsCount: 0,
            Mediums: core::ptr::null(),
            DataRangesCount: 1,
            DataRanges: BRIDGE_DATARANGES.as_ptr() as *const *mut KSDATAFORMAT,
            DataFlow: KSPIN_DATAFLOW_IN as i32,
            Communication: KSPIN_COMMUNICATION_NONE as i32, // INTERNAL (NONE)
            Category: &KSCATEGORY_AUDIO_GUID as *const GUID,
            Name: core::ptr::null(),
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
            InterfacesCount: 1,
            Interfaces: KSINTERFACES.as_ptr() as *const core::ffi::c_void,
            MediumsCount: 0,
            Mediums: core::ptr::null(),
            DataRangesCount: 1,
            DataRanges: BRIDGE_DATARANGES.as_ptr() as *const *mut KSDATAFORMAT,
            DataFlow: KSPIN_DATAFLOW_IN as i32,
            Communication: KSPIN_COMMUNICATION_NONE as i32, // INTERNAL (NONE)
            Category: &KSCATEGORY_AUDIO_GUID as *const GUID,
            Name: core::ptr::null(),
            Reserved: 0,
            Reserved2: 0,
        },
    },
    PCPIN_DESCRIPTOR {
        MaxGlobalInstanceCount: 1,
        MaxFilterInstanceCount: 1,
        MinFilterInstanceCount: 1,
        AutomationTable: &TOPO_PIN_AUTOMATION_TABLE,
        KsPinDescriptor: KSPIN_DESCRIPTOR {
            InterfacesCount: 1,
            Interfaces: KSINTERFACES.as_ptr() as *const core::ffi::c_void,
            MediumsCount: 0,
            Mediums: core::ptr::null(),
            DataRangesCount: 1,
            DataRanges: BRIDGE_DATARANGES.as_ptr() as *const *mut KSDATAFORMAT,
            DataFlow: KSPIN_DATAFLOW_OUT as i32,
            Communication: KSPIN_COMMUNICATION_BRIDGE as i32, // EXTERNAL BRIDGE (Speaker)
            Category: &KSNODETYPE_SPEAKER as *const GUID,
            Name: core::ptr::null(),
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
        AutomationTable: &TOPO_PIN_AUTOMATION_TABLE,
        KsPinDescriptor: KSPIN_DESCRIPTOR {
            InterfacesCount: 1,
            Interfaces: KSINTERFACES.as_ptr() as *const core::ffi::c_void,
            MediumsCount: 0,
            Mediums: core::ptr::null(),
            DataRangesCount: 1,
            DataRanges: BRIDGE_DATARANGES.as_ptr() as *const *mut KSDATAFORMAT,
            DataFlow: KSPIN_DATAFLOW_IN as i32,
            Communication: KSPIN_COMMUNICATION_BRIDGE as i32, // EXTERNAL BRIDGE (Mic)
            Category: &KSNODETYPE_MICROPHONE as *const GUID,
            Name: core::ptr::null(),
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
            InterfacesCount: 1,
            Interfaces: KSINTERFACES.as_ptr() as *const core::ffi::c_void,
            MediumsCount: 0,
            Mediums: core::ptr::null(),
            DataRangesCount: 1,
            DataRanges: BRIDGE_DATARANGES.as_ptr() as *const *mut KSDATAFORMAT,
            DataFlow: KSPIN_DATAFLOW_OUT as i32,
            Communication: KSPIN_COMMUNICATION_NONE as i32, // INTERNAL (NONE)
            Category: &KSCATEGORY_AUDIO_GUID as *const GUID,
            Name: core::ptr::null(),
            Reserved: 0,
            Reserved2: 0,
        },
    },
];

// ============================================================================
// Connections & Categories
// ============================================================================

// ============================================================================
// Property Handlers (Volume)
// ============================================================================

#[allow(clippy::missing_safety_doc)]
pub unsafe extern "C" fn volume_handler(property_request: PPCPROPERTY_REQUEST) -> NTSTATUS {
    if property_request.is_null() {
        return STATUS_INVALID_PARAMETER;
    }

    DbgPrint(c"Leyline: volume_handler CALLED\n".as_ptr());

    if (*property_request).ValueSize == 0 {
        (*property_request).ValueSize = core::mem::size_of::<i32>() as u32;
        return STATUS_BUFFER_OVERFLOW;
    }

    if (*property_request).ValueSize < core::mem::size_of::<i32>() as u32 {
        return STATUS_BUFFER_TOO_SMALL;
    }

    // Return 0dB (0x10000 normally, or 0 depending on scale)
    // KSPROPERTY_AUDIO_VOLUMELEVEL is usually logarithmic scale. 0 = full attenuation?
    // Let's check docs. 0x0 = 0dB? No.
    // Windows expects volume in steps.
    // Let's just return STATUS_SUCCESS and pretend we set it.
    // If it's a GET, return max volume.
    if (*property_request).Verb & KSPROPERTY_TYPE_GET != 0 {
        let value = (*property_request).Value as *mut i32;
        *value = 0; // 0 indicates 0dB usually? Or -infinity?
                    // Valid range is usually defined by BasicSupport.
    }

    STATUS_SUCCESS
}

#[link_section = ".rdata"]
pub static VOLUME_PROPERTIES: [PCPROPERTY_ITEM; 1] = [PCPROPERTY_ITEM {
    Set: &KSPROPSETID_Audio as *const GUID,
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
pub static TOPO_NODES: [crate::stream::PCNODE_DESCRIPTOR; 1] = [crate::stream::PCNODE_DESCRIPTOR {
    Flags: 0,
    AutomationTable: &VOLUME_AUTOMATION_TABLE,
    Type: &KSNODETYPE_VOLUME,
    Name: &GUID_NULL,
}];

// ============================================================================
// Connections & Categories
// ============================================================================

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
pub static TOPO_CONNECTIONS: [PCCONNECTION; 1] = [PCCONNECTION {
    FromNode: PCFILTER_NODE,
    FromNodePin: KSPIN_TOPO_BRIDGE,
    ToNode: PCFILTER_NODE,
    ToNodePin: KSPIN_TOPO_LINEOUT,
}];

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

// ============================================================================
// Filter Descriptors
// ============================================================================

#[link_section = ".rdata"]
pub static WAVE_RENDER_FILTER_DESCRIPTOR: PCFILTER_DESCRIPTOR = PCFILTER_DESCRIPTOR {
    Version: 0,
    AutomationTable: &COMPONENT_AUTOMATION_TABLE,
    PinSize: core::mem::size_of::<PCPIN_DESCRIPTOR>() as u32,
    PinCount: 2,
    Pins: WAVE_RENDER_PINS.as_ptr(),
    NodeSize: 0,
    NodeCount: 0,
    Nodes: core::ptr::null(),
    ConnectionCount: 0,
    Connections: core::ptr::null(),
    CategoryCount: 3,
    Categories: WAVE_RENDER_CATEGORIES.as_ptr(),
};

#[link_section = ".rdata"]
pub static WAVE_CAPTURE_FILTER_DESCRIPTOR: PCFILTER_DESCRIPTOR = PCFILTER_DESCRIPTOR {
    Version: 0,
    AutomationTable: &COMPONENT_AUTOMATION_TABLE,
    PinSize: core::mem::size_of::<PCPIN_DESCRIPTOR>() as u32,
    PinCount: 2,
    Pins: WAVE_CAPTURE_PINS.as_ptr(),
    NodeSize: 0,
    NodeCount: 0,
    Nodes: core::ptr::null(),
    ConnectionCount: 0,
    Connections: core::ptr::null(),
    CategoryCount: 3,
    Categories: WAVE_CAPTURE_CATEGORIES.as_ptr(),
};

#[link_section = ".rdata"]
pub static TOPO_RENDER_FILTER_DESCRIPTOR: PCFILTER_DESCRIPTOR = PCFILTER_DESCRIPTOR {
    Version: 0,
    AutomationTable: &COMPONENT_AUTOMATION_TABLE,
    PinSize: core::mem::size_of::<PCPIN_DESCRIPTOR>() as u32,
    PinCount: 2,
    Pins: TOPO_RENDER_PINS.as_ptr(),
    NodeSize: 0,
    NodeCount: 0,
    Nodes: core::ptr::null(),
    ConnectionCount: 1,
    Connections: TOPO_CONNECTIONS.as_ptr(),
    CategoryCount: 2,
    Categories: TOPO_FILTER_CATEGORIES.as_ptr(),
};

#[link_section = ".rdata"]
pub static TOPO_CAPTURE_FILTER_DESCRIPTOR: PCFILTER_DESCRIPTOR = PCFILTER_DESCRIPTOR {
    Version: 0,
    AutomationTable: &COMPONENT_AUTOMATION_TABLE,
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
