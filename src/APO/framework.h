// Copyright (c) 2026 Randall Rosas (Slategray).
// All rights reserved.
//
// This source code is provided for educational and review purposes.
// Redistribution and use in binary form without express permission is prohibited.
// See LICENSE file in the project root for full terms.

#ifndef LEYLINE_APO_FRAMEWORK_H_
#define LEYLINE_APO_FRAMEWORK_H_

#define WIN32_LEAN_AND_MEAN
#define STRICT

#include <windows.h>
#include <unknwn.h>
#include <objbase.h>
#include <tchar.h>

// Audio APO specific headers.
#include <audioenginebaseapo.h>
#include <audioengineextensionapo.h>
#include <baseaudioprocessingobject.h>

// Generated from IDL.
#include "LeylineAPO_h.h"

#endif  // LEYLINE_APO_FRAMEWORK_H_
