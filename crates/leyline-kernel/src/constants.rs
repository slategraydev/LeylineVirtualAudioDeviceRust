// Copyright (c) 2026 Randall Rosas (Slategray).
// All rights reserved.

// ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
// KERNEL CONSTANTS
// GUIDs and identifiers for ACX, KS formats, and device categories.
// ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

use wdk_sys::*;

// ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
// KS Formats & Specifiers (Framework-Agnostic)
// ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

#[allow(non_upper_case_globals)]
pub const KSDATAFORMAT_TYPE_AUDIO: GUID = GUID {
    Data1: 0x73647561,
    Data2: 0x0000,
    Data3: 0x0010,
    Data4: [0x80, 0x00, 0x00, 0xAA, 0x00, 0x38, 0x9B, 0x71],
};

#[allow(non_upper_case_globals)]
pub const KSDATAFORMAT_SUBTYPE_PCM: GUID = GUID {
    Data1: 0x00000001,
    Data2: 0x0000,
    Data3: 0x0010,
    Data4: [0x80, 0x00, 0x00, 0xAA, 0x00, 0x38, 0x9B, 0x71],
};

#[allow(non_upper_case_globals)]
pub const KSDATAFORMAT_SUBTYPE_IEEE_FLOAT: GUID = GUID {
    Data1: 0x00000003,
    Data2: 0x0000,
    Data3: 0x0010,
    Data4: [0x80, 0x00, 0x00, 0xAA, 0x00, 0x38, 0x9B, 0x71],
};

#[allow(non_upper_case_globals)]
pub const KSDATAFORMAT_SPECIFIER_WAVEFORMATEX: GUID = GUID {
    Data1: 0x05589F81,
    Data2: 0xC356,
    Data3: 0x11CE,
    Data4: [0xBF, 0x01, 0x00, 0xAA, 0x00, 0x55, 0x59, 0x5A],
};

// ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
// KS Node Types (Used by ACX for endpoint classification)
// ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

#[allow(non_upper_case_globals)]
pub static KSNODETYPE_SPEAKER: GUID = GUID {
    Data1: 0xDFF21CE1,
    Data2: 0xF70F,
    Data3: 0x11D0,
    Data4: [0xB9, 0x17, 0x00, 0xA0, 0xC9, 0x22, 0x31, 0x96],
};

#[allow(non_upper_case_globals)]
pub static KSNODETYPE_MICROPHONE: GUID = GUID {
    Data1: 0xDFF21BE1,
    Data2: 0xF70F,
    Data3: 0x11D0,
    Data4: [0xB9, 0x17, 0x00, 0xA0, 0xC9, 0x22, 0x31, 0x96],
};

// ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
// Filter & Pin Categories (Used by ACX for device interface registration)
// ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

#[allow(non_upper_case_globals)]
pub static KSCATEGORY_AUDIO_GUID: GUID = GUID {
    Data1: 0x6994AD04,
    Data2: 0x93EF,
    Data3: 0x11D0,
    Data4: [0xA3, 0xCC, 0x00, 0xA0, 0xC9, 0x22, 0x31, 0x96],
};

#[allow(non_upper_case_globals)]
pub static KSCATEGORY_RENDER_GUID: GUID = GUID {
    Data1: 0x65E8773E,
    Data2: 0x8F56,
    Data3: 0x11D0,
    Data4: [0xA3, 0xB9, 0x00, 0xA0, 0xC9, 0x22, 0x31, 0x96],
};

#[allow(non_upper_case_globals)]
pub static KSCATEGORY_CAPTURE_GUID: GUID = GUID {
    Data1: 0x65E8773D,
    Data2: 0x8F56,
    Data3: 0x11D0,
    Data4: [0xA3, 0xB9, 0x00, 0xA0, 0xC9, 0x22, 0x31, 0x96],
};

// ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
// Null GUID
// ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

#[allow(non_upper_case_globals)]
pub const GUID_NULL: GUID = GUID {
    Data1: 0x00000000,
    Data2: 0x0000,
    Data3: 0x0000,
    Data4: [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00],
};
