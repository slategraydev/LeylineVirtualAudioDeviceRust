#pragma once
#include "framework.h"

// ============================================================================
// CLeylineAPO
// ============================================================================
// This class implements the core Audio Processing Object for the Leyline driver.
// It inherits from CBaseAudioProcessingObject for boilerplate APO logic and
// implements ILeylineAPO for custom control.
class CLeylineAPO :
    public CBaseAudioProcessingObject,
    public ILeylineAPO,
    public IAudioSystemEffects
{
public:
    CLeylineAPO();
    virtual ~CLeylineAPO();

    // IUnknown methods
    STDMETHOD_(ULONG, AddRef)();
    STDMETHOD_(ULONG, Release)();
    STDMETHOD(QueryInterface)(REFIID riid, void** ppvObject);

    // IAudioProcessingObjectRT methods
    STDMETHOD_(void, APOProcess)(
        UINT32 u32NumInputConnections,
        APO_CONNECTION_PROPERTY** ppInputConnections,
        UINT32 u32NumOutputConnections,
        APO_CONNECTION_PROPERTY** ppOutputConnections);

    // IAudioProcessingObject methods
    STDMETHOD(Initialize)(UINT32 cbDataSize, BYTE* pbyData);
    STDMETHOD(IsInputFormatSupported)(
        IAudioMediaType* pOppositeFormat,
        IAudioMediaType* pRequestedInputFormat,
        IAudioMediaType** ppSupportedInputFormat);

private:
    LONG m_cRef;
    HANDLE m_hDriver;
    float* m_pSharedBuffer;
    void* m_pSharedParams; // Pointer to SharedParameters struct
    float m_fGain;
    float m_fPeakL;
    float m_fPeakR;

    void UpdatePeakMeter(float left, float right);
};
