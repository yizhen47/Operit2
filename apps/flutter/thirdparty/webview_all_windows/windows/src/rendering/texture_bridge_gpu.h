#pragma once

#include <d3d11.h>
#include <dxgi.h>

#include <atomic>

#include <flutter/texture_registrar.h>

#include "rendering/texture_bridge.h"

namespace webview_all_windows {

class TextureBridgeGpu : public TextureBridge {
public:
  TextureBridgeGpu(GraphicsContext *graphics_context,
                   ABI::Windows::UI::Composition::IVisual *visual);

  const FlutterDesktopGpuSurfaceDescriptor *GetSurfaceDescriptor(size_t width,
                                                                 size_t height);

protected:
  /// Copies the capture texture into the back buffer and publishes that buffer.
  bool PublishCapturedTexture(ID3D11Texture2D *texture) override;

  /// Keeps the published slot from being overwritten during CPU readback.
  std::function<void()> RetainReadback() override;

private:
  struct SharedSlot {
    winrt::com_ptr<ID3D11Texture2D> texture;
    winrt::com_ptr<IDXGIResource> dxgi;
    HANDLE handle = nullptr;
    uint32_t width = 0;
    uint32_t height = 0;
    std::atomic<int> readbacks{0};
  };

  static constexpr int kSharedSlotCount = 2;

  FlutterDesktopGpuSurfaceDescriptor surface_descriptor_ = {};
  SharedSlot slots_[kSharedSlotCount];
  int published_index_ = -1;

  /// Creates or resizes one shared DXGI texture used as a Flutter GPU surface.
  bool EnsureSlot(SharedSlot *slot, uint32_t width, uint32_t height);
};

} // namespace webview_all_windows
