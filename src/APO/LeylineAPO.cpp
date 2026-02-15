#include "LeylineAPO.h"
#include <initguid.h>
#include <math.h>

// {C8D3E4F5-B6A7-4A2D-A1A3-1A2B3C4D5E6F}
DEFINE_GUID(CLSID_LeylineAPO, 0xc8d3e4f5, 0xb6a7, 0x4a2d, 0xa1, 0xa3, 0x1a, 0x2b, 0x3c, 0x4d, 0x5e, 0x6f);

// {D9A2A1A3-C7B1-4A2D-1A2B-3C4D5E6F77B8}
DEFINE_GUID(IID_ILeylineAPO, 0xd9a2a1a3, 0xc7b1, 0x4a2d, 0x1a, 0x2b, 0x3c, 0x4d, 0x5e, 0x6f, 0x77, 0xb8);

// ============================================================================
// Shared Memory & Helpers
// ============================================================================

#define IOCTL_LEYLINE_MAP_BUFFER 0x80002008
#define IOCTL_LEYLINE_MAP_PARAMS 0x8000200C

struct SharedParameters {
    LONG master_gain_bits;
    LONG peak_l_bits;
    LONG peak_r_bits;
};

// Bit-cast helpers for atomic operations
LONG FloatToLong(float f) {
    union { float f; LONG l; } u;
    u.f = f;
    return u.l;
}

float LongToFloat(LONG l) {
    union { float f; LONG l; } u;
    u.l = l;
    return u.f;
}

// ============================================================================
// Implementation
// ============================================================================

CLeylineAPO::CLeylineAPO() :
    m_cRef(1),
    m_hDriver(INVALID_HANDLE_VALUE),
    m_pSharedBuffer(NULL),
    m_pSharedParams(NULL),
    m_fGain(1.0f),
    m_fPeakL(0.0f),
    m_fPeakR(0.0f)
{
    // Initialize registration properties
    // We strictly use the default flags (APO_FLAG_DEFAULT) as we process data in-place.
    ZeroMemory(&m_RegProperties, sizeof(m_RegProperties));
    m_RegProperties.clsid = CLSID_LeylineAPO;
    m_RegProperties.Flags = APO_FLAG_DEFAULT;
    // Standard APOs typically support 1 input and 1 output connection.
    m_RegProperties.u32MinInputConnections = 1;
    m_RegProperties.u32MaxInputConnections = 1;
    m_RegProperties.u32MinOutputConnections = 1;
    m_RegProperties.u32MaxOutputConnections = 1;
}

CLeylineAPO::~CLeylineAPO()
{
    if (m_hDriver != INVALID_HANDLE_VALUE)
    {
        CloseHandle(m_hDriver);
    }
}

// ============================================================================
// IUnknown
// ============================================================================

STDMETHODIMP_(ULONG) CLeylineAPO::AddRef()
{
    return InterlockedIncrement(&m_cRef);
}

STDMETHODIMP_(ULONG) CLeylineAPO::Release()
{
    ULONG cRef = InterlockedDecrement(&m_cRef);
    if (0 == cRef)
    {
        delete this;
    }
    return cRef;
}

STDMETHODIMP CLeylineAPO::QueryInterface(REFIID riid, void** ppvObject)
{
    if (NULL == ppvObject)
    {
        return E_POINTER;
    }

    if (riid == IID_IUnknown)
    {
        *ppvObject = static_cast<IUnknown*>(static_cast<ILeylineAPO*>(this));
    }
    else if (riid == IID_ILeylineAPO)
    {
        *ppvObject = static_cast<ILeylineAPO*>(this);
    }
    else if (riid == __uuidof(IAudioProcessingObject))
    {
        *ppvObject = static_cast<IAudioProcessingObject*>(this);
    }
    else if (riid == __uuidof(IAudioProcessingObjectRT))
    {
        *ppvObject = static_cast<IAudioProcessingObjectRT*>(this);
    }
    else if (riid == __uuidof(IAudioSystemEffects))
    {
        *ppvObject = static_cast<IAudioSystemEffects*>(this);
    }
    else
    {
        *ppvObject = NULL;
        return E_NOINTERFACE;
    }

    AddRef();
    return S_OK;
}

// ============================================================================
// IAudioProcessingObjectRT
// ============================================================================

STDMETHODIMP_(void) CLeylineAPO::APOProcess(
    UINT32 u32NumInputConnections,
    APO_CONNECTION_PROPERTY** ppInputConnections,
    UINT32 u32NumOutputConnections,
    APO_CONNECTION_PROPERTY** ppOutputConnections)
{
    if (u32NumInputConnections == 1 && u32NumOutputConnections == 1)
    {
        float* pfIn = (float*)ppInputConnections[0]->pBuffer;
        float* pfOut = (float*)ppOutputConnections[0]->pBuffer;
        UINT32 u32Frames = ppInputConnections[0]->u32ValidFrameCount;

        m_fPeakL = 0.0f;
        m_fPeakR = 0.0f;

        for (UINT32 i = 0; i < u32Frames; i++)
        {
            // Apply gain
            float sampleL = pfIn[i * 2] * m_fGain;
            float sampleR = pfIn[i * 2 + 1] * m_fGain;

            pfOut[i * 2] = sampleL;
            pfOut[i * 2 + 1] = sampleR;

            // Simple peak detection
            UpdatePeakMeter(sampleL, sampleR);
        }

        // Copy to shared buffer for HSA monitoring
        if (m_pSharedBuffer)
        {
            CopyMemory(m_pSharedBuffer, pfOut, u32Frames * sizeof(float) * 2);
        }

        if (m_pSharedParams)
        {
            SharedParameters* pParams = (SharedParameters*)m_pSharedParams;
            // Atomic Read: Use InterlockedOr to read 32-bit value atomically
            LONG lGain = InterlockedOr(&pParams->master_gain_bits, 0);
            m_fGain = LongToFloat(lGain);

            // Atomic Write: Use InterlockedExchange to update peaks preventing tearing
            InterlockedExchange(&pParams->peak_l_bits, FloatToLong(m_fPeakL));
            InterlockedExchange(&pParams->peak_r_bits, FloatToLong(m_fPeakR));
        }

        ppOutputConnections[0]->u32ValidFrameCount = u32Frames;
        ppOutputConnections[0]->u32BufferFlags = ppInputConnections[0]->u32BufferFlags;
    }
}

STDMETHODIMP_(UINT32) CLeylineAPO::CalcInputFrames(UINT32 u32OutputFrameCount) {
    return u32OutputFrameCount;
}

STDMETHODIMP_(UINT32) CLeylineAPO::CalcOutputFrames(UINT32 u32InputFrameCount) {
    return u32InputFrameCount;
}

void CLeylineAPO::UpdatePeakMeter(float left, float right)
{
    float absL = fabsf(left);
    float absR = fabsf(right);
    if (absL > m_fPeakL) m_fPeakL = absL;
    if (absR > m_fPeakR) m_fPeakR = absR;
}

// ============================================================================
// IAudioProcessingObject
// ============================================================================

STDMETHODIMP CLeylineAPO::Initialize(UINT32 cbDataSize, BYTE* pbyData)
{
    UNREFERENCED_PARAMETER(cbDataSize);
    UNREFERENCED_PARAMETER(pbyData);
    
    // Open the driver for real-time communication
    m_hDriver = CreateFile(L"\\\\.\\LeylineAudio", GENERIC_READ | GENERIC_WRITE, 
                           0, NULL, OPEN_EXISTING, FILE_ATTRIBUTE_NORMAL, NULL);

    if (m_hDriver != INVALID_HANDLE_VALUE)
    {
        DWORD dwBytesReturned = 0;
        PVOID pUserMapping = NULL;

        // Request the shared buffer mapping from the kernel driver
        if (DeviceIoControl(m_hDriver, IOCTL_LEYLINE_MAP_BUFFER, NULL, 0, 
                             &pUserMapping, sizeof(PVOID), &dwBytesReturned, NULL))
        {
            m_pSharedBuffer = (float*)pUserMapping;
        }

        // Request the shared parameter mapping
        if (DeviceIoControl(m_hDriver, IOCTL_LEYLINE_MAP_PARAMS, NULL, 0,
                             &pUserMapping, sizeof(PVOID), &dwBytesReturned, NULL))
        {
            m_pSharedParams = pUserMapping;
        }
    }

    return S_OK;
}

STDMETHODIMP CLeylineAPO::Reset() {
    return S_OK;
}

STDMETHODIMP CLeylineAPO::GetLatency(HNSTIME* pTime) {
    if (!pTime) return E_POINTER;
    *pTime = 0;
    return S_OK;
}

STDMETHODIMP CLeylineAPO::GetRegistrationProperties(APO_REG_PROPERTIES** ppRegProps) {
    if (!ppRegProps) return E_POINTER;

    // Allocate the registration properties structure using COM task memory.
    // The audio engine is responsible for freeing this memory.
    APO_REG_PROPERTIES* pProps = (APO_REG_PROPERTIES*)CoTaskMemAlloc(sizeof(APO_REG_PROPERTIES));
    if (!pProps) return E_OUTOFMEMORY;

    *pProps = m_RegProperties;
    *ppRegProps = pProps;
    return S_OK;
}

STDMETHODIMP CLeylineAPO::GetInputChannelCount(UINT32* pu32ChannelCount) {
    if (!pu32ChannelCount) return E_POINTER;
    *pu32ChannelCount = 2; // Stereo
    return S_OK;
}

STDMETHODIMP CLeylineAPO::IsOutputFormatSupported(
    IAudioMediaType* pOppositeFormat,
    IAudioMediaType* pRequestedOutputFormat,
    IAudioMediaType** ppSupportedOutputFormat)
{
    // Mirror the input logic for now - we are 1:1 in/out
    return IsInputFormatSupported(pOppositeFormat, pRequestedOutputFormat, ppSupportedOutputFormat);
}

STDMETHODIMP CLeylineAPO::IsInputFormatSupported(
    IAudioMediaType* pOppositeFormat,
    IAudioMediaType* pRequestedInputFormat,
    IAudioMediaType** ppSupportedInputFormat)
{
    if (NULL == pRequestedInputFormat)
    {
        return E_POINTER;
    }

    // Helper to validate format
    auto IsFloat32Stereo = [](IAudioMediaType* pFormat) -> bool {
        const WAVEFORMATEX* pWfx = pFormat->GetAudioFormat();
        if (NULL == pWfx) return false;

        if (pWfx->wFormatTag == WAVE_FORMAT_IEEE_FLOAT)
        {
            return (pWfx->nChannels == 2 && pWfx->wBitsPerSample == 32);
        }
        else if (pWfx->wFormatTag == WAVE_FORMAT_EXTENSIBLE)
        {
            const WAVEFORMATEXTENSIBLE* pWfxExt = (const WAVEFORMATEXTENSIBLE*)pWfx;
            return (pWfx->nChannels == 2 && 
                    pWfx->wBitsPerSample == 32 &&
                    IsEqualGUID(pWfxExt->SubFormat, KSDATAFORMAT_SUBTYPE_IEEE_FLOAT));
        }
        return false;
    };

    // 1. If pOppositeFormat is provided, the requested format must match it conceptually
    //    (Unless we implemented sample rate conversion, which we haven't).
    if (pOppositeFormat)
    {
        if (!IsFloat32Stereo(pOppositeFormat))
        {
            return S_FALSE; // We only support Float32 Stereo output too
        }

        // For now, strict passthrough match
        if (pOppositeFormat->IsEqual(pRequestedInputFormat, NULL) != S_OK)
        {
             return S_FALSE;
        }
    }

    // 2. Validate the requested input format itself
    if (IsFloat32Stereo(pRequestedInputFormat))
    {
        return S_OK;
    }

    // 3. If we get here, the format is NOT supported. 
    //    We should propose a supported format if asked.
    if (ppSupportedInputFormat)
    {
        // For brevity in this session, we won't construct a full media type from scratch
        // as it requires CoCreateInstance(CLSID_AudioMediaType) which might not be 
        // trivially available without boilerplate. 
        // Returning S_FALSE with *ppSupportedInputFormat = NULL is valid but less helpful.
        *ppSupportedInputFormat = NULL; 
    }

    return S_FALSE;
}
