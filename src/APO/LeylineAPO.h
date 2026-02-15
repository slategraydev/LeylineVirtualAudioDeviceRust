#pragma once
#include "framework.h"

// ============================================================================
// CLeylineAPO
// ============================================================================
// This class implements the core Audio Processing Object for the Leyline driver.
// It inherits directly from the required COM interfaces.
class CLeylineAPO :
    public ILeylineAPO,
    public IAudioProcessingObject,
    public IAudioProcessingObjectRT,
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

    STDMETHOD_(UINT32, CalcInputFrames)(UINT32 u32OutputFrameCount);
    STDMETHOD_(UINT32, CalcOutputFrames)(UINT32 u32InputFrameCount);

    // IAudioProcessingObject methods
    STDMETHOD(Initialize)(UINT32 cbDataSize, BYTE* pbyData);
    STDMETHOD(IsInputFormatSupported)(
        IAudioMediaType* pOppositeFormat,
        IAudioMediaType* pRequestedInputFormat,
        IAudioMediaType** ppSupportedInputFormat);
    STDMETHOD(IsOutputFormatSupported)(
        IAudioMediaType* pOppositeFormat,
        IAudioMediaType* pRequestedOutputFormat,
        IAudioMediaType** ppSupportedOutputFormat);
    STDMETHOD(GetRegistrationProperties)(APO_REG_PROPERTIES** ppRegProps);
    STDMETHOD(GetInputChannelCount)(UINT32* pu32ChannelCount);
    STDMETHOD(Reset)();
    STDMETHOD(GetLatency)(HNSTIME* pTime);

    // IAudioSystemEffects methods (Stub)
    // (Usually handles valid/invalid state) - assuming minimal requirement here or IUnknown sufficiency.
    // If specific methods are needed for IAudioSystemEffects beyond IUnknown, they'd go here. 
    // Note: IAudioSystemEffects mostly inherits IUnknown.

private:
    LONG m_cRef;
    HANDLE m_hDriver;
    float* m_pSharedBuffer;
    void* m_pSharedParams;
    float m_fGain;
    float m_fPeakL;
    float m_fPeakR;
    
    // Stub properties
    APO_REG_PROPERTIES m_RegProperties;

    void UpdatePeakMeter(float left, float right);
};
