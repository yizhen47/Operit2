#include "rendering/texture_bridge_gpu.h"

#include "util/direct3d11.interop.h"
#include "util/logging.h"

namespace webview_all_windows {

TextureBridgeGpu::TextureBridgeGpu(
    GraphicsContext *graphics_context,
    ABI::Windows::UI::Composition::IVisual *visual)
    : TextureBridge(graphics_context, visual) {
  surface_descriptor_.struct_size = sizeof(FlutterDesktopGpuSurfaceDescriptor);
  surface_descriptor_.format =
      kFlutterDesktopPixelFormatNone; // no format required for DXGI surfaces
}

/// Copies one capture frame into the shared slot Flutter is not sampling.
bool TextureBridgeGpu::PublishCapturedTexture(ID3D11Texture2D *texture) {
  if (!texture) {
    return false;
  }

  D3D11_TEXTURE2D_DESC desc;
  texture->GetDesc(&desc);
  const int write_index =
      published_index_ < 0 ? 0 : (published_index_ ^ 1);
  SharedSlot *slot = &slots_[write_index];
  if (slot->readbacks.load() != 0) {
    return false;
  }
  if (!EnsureSlot(slot, desc.Width, desc.Height)) {
    return false;
  }

  graphics_context_->d3d_device_context()->CopyResource(slot->texture.get(),
                                                        texture);
  published_index_ = write_index;
  last_frame_ = slot->texture;
  return true;
}

/// Holds the published slot until the queued CPU readback releases it.
std::function<void()> TextureBridgeGpu::RetainReadback() {
  if (published_index_ < 0) {
    return {};
  }
  SharedSlot *slot = &slots_[published_index_];
  slot->readbacks.fetch_add(1);
  return [slot]() { slot->readbacks.fetch_sub(1); };
}

/// Allocates a shared BGRA texture for one Flutter-sampled frame slot.
bool TextureBridgeGpu::EnsureSlot(SharedSlot *slot, uint32_t width,
                                  uint32_t height) {
  if (slot->texture && slot->width == width && slot->height == height) {
    return true;
  }
  if (slot->readbacks.load() != 0) {
    return false;
  }

  D3D11_TEXTURE2D_DESC dst_desc = {};
  dst_desc.ArraySize = 1;
  dst_desc.MipLevels = 1;
  dst_desc.BindFlags = D3D11_BIND_RENDER_TARGET | D3D11_BIND_SHADER_RESOURCE;
  dst_desc.CPUAccessFlags = 0;
  dst_desc.Format = static_cast<DXGI_FORMAT>(kPixelFormat);
  dst_desc.Width = width;
  dst_desc.Height = height;
  dst_desc.MiscFlags = D3D11_RESOURCE_MISC_SHARED;
  dst_desc.SampleDesc.Count = 1;
  dst_desc.SampleDesc.Quality = 0;
  dst_desc.Usage = D3D11_USAGE_DEFAULT;

  winrt::com_ptr<ID3D11Texture2D> created;
  if (!SUCCEEDED(graphics_context_->d3d_device()->CreateTexture2D(
          &dst_desc, nullptr, created.put()))) {
    util::LogWarning("Creating intermediate texture failed.");
    return false;
  }

  winrt::com_ptr<IDXGIResource> dxgi;
  created.try_as(dxgi);
  assert(dxgi);
  HANDLE shared_handle = nullptr;
  dxgi->GetSharedHandle(&shared_handle);

  slot->texture = std::move(created);
  slot->dxgi = std::move(dxgi);
  slot->handle = shared_handle;
  slot->width = width;
  slot->height = height;
  return true;
}

/// Returns the published shared surface without copying on the raster thread.
const FlutterDesktopGpuSurfaceDescriptor *
TextureBridgeGpu::GetSurfaceDescriptor(size_t width, size_t height) {
  (void)width;
  (void)height;
  const std::lock_guard<std::mutex> lock(mutex_);
  if (published_index_ < 0) {
    return nullptr;
  }

  SharedSlot *slot = &slots_[published_index_];
  if (!slot->texture) {
    return nullptr;
  }

  surface_descriptor_.handle = slot->handle;
  surface_descriptor_.width = surface_descriptor_.visible_width = slot->width;
  surface_descriptor_.height = surface_descriptor_.visible_height =
      slot->height;
  surface_descriptor_.release_context = slot->texture.get();
  surface_descriptor_.release_callback = [](void *release_context) {
    auto texture = reinterpret_cast<ID3D11Texture2D *>(release_context);
    texture->Release();
  };

  // The engine releases this reference after it binds the shared handle.
  slot->texture->AddRef();
  return &surface_descriptor_;
}

} // namespace webview_all_windows
