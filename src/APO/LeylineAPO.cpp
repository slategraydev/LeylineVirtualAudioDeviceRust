#include "LeylineAPO.h"
#include <initguid.h>

// {C8D3E4F5-B6A7-4A2D-A1A3-1A2B3C4D5E6F}
DEFINE_GUID(CLSID_LeylineAPO, 0xc8d3e4f5, 0xb6a7, 0x4a2d, 0xa1, 0xa3, 0x1a, 0x2b, 0x3c, 0x4d, 0x5e, 0x6f);

// {D9A2A1A3-C7B1-4A2D-1A2B-3C4D5E6F77B8}
DEFINE_GUID(IID_ILeylineAPO, 0xd9a2a1a3, 0xc7b1, 0x4a2d, 0x1a, 0x2b, 0x3c, 0x4d, 0x5e, 0x6f, 0x77, 0xb8);

CLeylineAPO::CLeylineAPO() :
    m_cRef(1),
    m_hDriver(INVALID_HANDLE_VALUE),
    m_pSharedBuffer(NULL),
    m_pSharedParams(NULL),
    m_fGain(1.0f),
    m_fPeakL(0.0f),
    m_fPeakR(0.0f)
{
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

    if (riid == IID_IUnknown || riid == __uuidof(IAudioProcessingObject) || riid == __uuidof(IAudioProcessingObjectRT))
    {
        *ppvObject = static_cast<IAudioProcessingObject*>(this);
    }
    else if (riid == IID_ILeylineAPO)
    {
        *ppvObject = static_cast<ILeylineAPO*>(this);
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
            m_fGain = pParams->master_gain;
            pParams->peak_l = m_fPeakL;
            pParams->peak_r = m_fPeakR;
        }

        ppOutputConnections[0]->u32ValidFrameCount = u32Frames;
        ppOutputConnections[0]->u32BufferFlags = ppInputConnections[0]->u32BufferFlags;
    }
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

#define IOCTL_LEYLINE_MAP_BUFFER 0x80002008
#define IOCTL_LEYLINE_MAP_PARAMS 0x8000200C

struct SharedParameters {
    float master_gain;
    float peak_l;
    float peak_r;
};

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

STDMETHODIMP CLeylineAPO::IsInputFormatSupported(
    IAudioMediaType* pOppositeFormat,
    IAudioMediaType* pRequestedInputFormat,
    IAudioMediaType** ppSupportedInputFormat)
{
    // Accept the requested input format by default for this boilerplate.
    if (ppSupportedInputFormat)
    {
        *ppSupportedInputFormat = NULL;
    }

    return S_OK;
}
