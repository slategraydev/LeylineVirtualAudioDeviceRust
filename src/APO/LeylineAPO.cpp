#include "LeylineAPO.h"
#include <initguid.h>

// {C8D3E4F5-B6A7-4A2D-A1A3-1A2B3C4D5E6F}
DEFINE_GUID(CLSID_LeylineAPO, 0xc8d3e4f5, 0xb6a7, 0x4a2d, 0xa1, 0xa3, 0x1a, 0x2b, 0x3c, 0x4d, 0x5e, 0x6f);

// {D9A2A1A3-C7B1-4A2D-1A2B-3C4D5E6F77B8}
DEFINE_GUID(IID_ILeylineAPO, 0xd9a2a1a3, 0xc7b1, 0x4a2d, 0x1a, 0x2b, 0x3c, 0x4d, 0x5e, 0x6f, 0x77, 0xb8);

CLeylineAPO::CLeylineAPO() :
    m_cRef(1)
{
}

CLeylineAPO::~CLeylineAPO()
{
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
    // For now, implement a simple bit-perfect pass-through.
    // In a real APO, this is where the Digital Signal Processing happens.
    if (u32NumInputConnections == 1 && u32NumOutputConnections == 1)
    {
        CopyMemory(
            ppOutputConnections[0]->pBuffer,
            ppInputConnections[0]->pBuffer,
            ppInputConnections[0]->u32ValidFrameCount * sizeof(float) * 2 // Assuming stereo float for now
        );
        ppOutputConnections[0]->u32ValidFrameCount = ppInputConnections[0]->u32ValidFrameCount;
        ppOutputConnections[0]->u32BufferFlags = ppInputConnections[0]->u32BufferFlags;
    }
}

// ============================================================================
// IAudioProcessingObject
// ============================================================================

STDMETHODIMP CLeylineAPO::Initialize(UINT32 cbDataSize, BYTE* pbyData)
{
    UNREFERENCED_PARAMETER(cbDataSize);
    UNREFERENCED_PARAMETER(pbyData);
    
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
