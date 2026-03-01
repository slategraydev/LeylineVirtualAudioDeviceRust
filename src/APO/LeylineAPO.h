// Copyright (c) 2026 Randall Rosas (Slategray).
// All rights reserved.

// ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
// APO DEFINITIONS
// Interface declarations and GUIDs for the Leyline APO.
// ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

#ifndef LEYLINE_APO_LEYLINE_APO_H_
#define LEYLINE_APO_LEYLINE_APO_H_

#include "framework.h"

namespace audio_apo {

// ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
// INTERFACE DEFINITION
// ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

// This class implements the core Audio Processing Object for the Leyline driver.
// It inherits directly from the required COM interfaces.
class CLeylineAPO : public ILeylineAPO,
                    public IAudioProcessingObject,
                    public IAudioProcessingObjectRT,
                    public IAudioSystemEffects {
 public:
  CLeylineAPO();
  virtual ~CLeylineAPO();

  // IUnknown methods.
  STDMETHOD_(ULONG, AddRef)();
  STDMETHOD_(ULONG, Release)();
  STDMETHOD(QueryInterface)(REFIID riid, void** ppvObject);

  // IAudioProcessingObjectRT methods.
  STDMETHOD_(void, APOProcess)
  (UINT32 u32NumInputConnections, APO_CONNECTION_PROPERTY** ppInputConnections,
   UINT32 u32NumOutputConnections,
   APO_CONNECTION_PROPERTY** ppOutputConnections);

  STDMETHOD_(UINT32, CalcInputFrames)(UINT32 u32OutputFrameCount);
  STDMETHOD_(UINT32, CalcOutputFrames)(UINT32 u32InputFrameCount);

  // IAudioProcessingObject methods.
  STDMETHOD(Initialize)(UINT32 cbDataSize, BYTE* pbyData);
  STDMETHOD(IsInputFormatSupported)
  (IAudioMediaType* pOppositeFormat, IAudioMediaType* pRequestedInputFormat,
   IAudioMediaType** ppSupportedInputFormat);
  STDMETHOD(IsOutputFormatSupported)
  (IAudioMediaType* pOppositeFormat, IAudioMediaType* pRequestedOutputFormat,
   IAudioMediaType** ppSupportedOutputFormat);
  STDMETHOD(GetRegistrationProperties)(APO_REG_PROPERTIES** ppRegProps);
  STDMETHOD(GetInputChannelCount)(UINT32* pu32ChannelCount);
  STDMETHOD(Reset)();
  STDMETHOD(GetLatency)(HNSTIME* pTime);

  // IAudioSystemEffects methods (Stub).
 private:
  LONG m_cRef;
  HANDLE m_hDriver;
  float* m_pSharedBuffer;
  void* m_pSharedParams;
  float m_fGain;
  float m_fPeakL;
  float m_fPeakR;

  // Stub properties.
  APO_REG_PROPERTIES m_RegProperties;

  void UpdatePeakMeter(float left, float right);
};

}  // namespace audio_apo

#endif  // LEYLINE_APO_LEYLINE_APO_H_
