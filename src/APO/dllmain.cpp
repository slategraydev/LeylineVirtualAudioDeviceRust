// Copyright (c) 2026 Randall Rosas (Slategray).
// All rights reserved.

#include <initguid.h>

#include "LeylineAPO.h"

namespace audio_apo {

HMODULE g_hModule = nullptr;

}  // namespace audio_apo

// {C8D3E4F5-B6A7-4A2D-A1A3-1A2B3C4D5E6F}
DEFINE_GUID(CLSID_LeylineAPO, 0xc8d3e4f5, 0xb6a7, 0x4a2d, 0xa1, 0xa3, 0x1a,
            0x2b, 0x3c, 0x4d, 0x5e, 0x6f);

// ===========================================================================
// CAPOClassFactory
// ===========================================================================
namespace audio_apo {
class CAPOClassFactory : public IClassFactory {
 public:
  CAPOClassFactory() : m_cRef(1) {}

  // IUnknown methods.
  STDMETHODIMP QueryInterface(REFIID riid, void** ppv) {
    if (riid == IID_IUnknown || riid == IID_IClassFactory) {
      *ppv = static_cast<IClassFactory*>(this);
      AddRef();
      return S_OK;
    }
    *ppv = nullptr;
    return E_NOINTERFACE;
  }

  STDMETHODIMP_(ULONG) AddRef() { return InterlockedIncrement(&m_cRef); }
  STDMETHODIMP_(ULONG) Release() {
    ULONG cRef = InterlockedDecrement(&m_cRef);
    if (cRef == 0) delete this;
    return cRef;
  }

  // IClassFactory methods.
  STDMETHODIMP CreateInstance(IUnknown* pUnkOuter, REFIID riid, void** ppv) {
    if (pUnkOuter != nullptr) return CLASS_E_NOAGGREGATION;
    CLeylineAPO* pAPO = new CLeylineAPO();
    if (pAPO == nullptr) return E_OUTOFMEMORY;
    HRESULT hr = pAPO->QueryInterface(riid, ppv);
    pAPO->Release();
    return hr;
  }

  STDMETHODIMP LockServer(BOOL fLock) {
    (void)fLock;
    return S_OK;
  }

 private:
  LONG m_cRef;
};

}  // namespace audio_apo

// ===========================================================================
// DLL Entry Points
// ===========================================================================

using namespace audio_apo;

BOOL APIENTRY DllMain(HMODULE hModule, DWORD ul_reason_for_call,
                      LPVOID lpReserved) {
  (void)lpReserved;
  switch (ul_reason_for_call) {
    case DLL_PROCESS_ATTACH:
      g_hModule = hModule;
      DisableThreadLibraryCalls(hModule);
      break;
  }
  return TRUE;
}

_Check_return_
STDAPI DllGetClassObject(_In_ REFCLSID rclsid, _In_ REFIID riid,
                         _Outptr_ LPVOID FAR* ppv) {
  if (rclsid == CLSID_LeylineAPO) {
    CAPOClassFactory* pFactory = new CAPOClassFactory();
    if (pFactory == nullptr) return E_OUTOFMEMORY;
    HRESULT hr = pFactory->QueryInterface(riid, ppv);
    pFactory->Release();
    return hr;
  }
  return CLASS_E_CLASSNOTAVAILABLE;
}

__control_entrypoint(DllExport) STDAPI DllCanUnloadNow(void) { return S_OK; }
