use super::*;

/// Flags for renderer creation.
///
/// See [Window::create_renderer](Window::create_renderer]
#[derive(Debug, Default, Clone, Copy)]
#[repr(transparent)]
pub struct RendererFlags(pub(crate) SDL_RendererFlags::Type);
#[allow(bad_style)]
type SDL_RendererFlags_Type = SDL_RendererFlags::Type;
#[allow(missing_docs)]
impl RendererFlags {
  phantom_fields! {
    self.0: SDL_RendererFlags_Type,
    accelerated: SDL_RENDERER_ACCELERATED,
    present_vsync: SDL_RENDERER_PRESENTVSYNC,
    software: SDL_RENDERER_SOFTWARE,
    target_texture: SDL_RENDERER_TARGETTEXTURE,
  }
}

/// Handle to some SDL2 rendering state.
///
/// Helps you do things like upload data to the GPU and blit image data around.
///
/// **To be clear: This is not a super fast renderer.** It's easy to use and you
/// can get an image on the screen, but if you want do much at all that's
/// computationally expensive you'll need to use a proper hardware API (OpenGL,
/// Vulkan, etc). Also, you cannot really mix this renderer with the hardware
/// APIs. They both expect to have full control of the pixel process. Use this
/// _or_ a hardware API.
#[derive(Debug)]
#[repr(transparent)]
pub struct Renderer<'sdl, 'win> {
  pub(crate) ptr: *mut SDL_Renderer,
  pub(crate) _marker: PhantomData<&'win Window<'sdl>>,
}
impl<'sdl, 'win> Drop for Renderer<'sdl, 'win> {
  fn drop(&mut self) {
    unsafe { SDL_DestroyRenderer(self.ptr) }
  }
}
impl<'sdl, 'win> Renderer<'sdl, 'win> {
  /// Makes a texture with the contents of the surface specified.
  ///
  /// The TextureAccess hint for textures from this is "static".
  ///
  /// The pixel format might be different from the surface's pixel format.
  pub fn create_texture_from_surface<'ren>(
    &'ren self, surf: &Surface,
  ) -> Result<Texture<'sdl, 'win, 'ren>, String> {
    let ptr: *mut SDL_Texture = unsafe { SDL_CreateTextureFromSurface(self.ptr, surf.ptr) };
    if ptr.is_null() {
      Err(get_error())
    } else {
      Ok(Texture {
        ptr,
        _marker: PhantomData,
      })
    }
  }

  /// Obtains the current draw color.
  pub fn draw_color(&self) -> Result<Color, String> {
    let mut color = Color::default();
    let out = unsafe {
      SDL_GetRenderDrawColor(
        self.ptr,
        &mut color.r,
        &mut color.g,
        &mut color.b,
        &mut color.a,
      )
    };
    if out == 0 {
      Ok(color)
    } else {
      Err(get_error())
    }
  }

  /// Assigns the color used for drawing.
  pub fn set_draw_color(&self, color: Color) -> Result<(), String> {
    let out = unsafe { SDL_SetRenderDrawColor(self.ptr, color.r, color.g, color.b, color.a) };
    if out == 0 {
      Ok(())
    } else {
      Err(get_error())
    }
  }

  /// Clears the render target with the current draw color.
  pub fn clear(&self) -> Result<(), String> {
    if unsafe { SDL_RenderClear(self.ptr) } == 0 {
      Ok(())
    } else {
      Err(get_error())
    }
  }

  /// Draws a line that includes both end points.
  pub fn draw_line(&self, x1: i32, y1: i32, x2: i32, y2: i32) -> Result<(), String> {
    let out = unsafe { SDL_RenderDrawLine(self.ptr, x1, y1, x2, y2) };
    if out == 0 {
      Ok(())
    } else {
      Err(get_error())
    }
  }

  /// Using the slice of `n` points, draws `n-1` lines end to end.
  pub fn draw_lines(&self, points: &[Point]) -> Result<(), String> {
    if points.len() > core::i32::MAX as usize {
      return Err("beryllium error: len cannot exceed `i32::MAX`.".to_string());
    }
    let ptr = points.as_ptr() as *const SDL_Point;
    let count = points.len() as i32;
    let out = unsafe { SDL_RenderDrawLines(self.ptr, ptr, count) };
    if out == 0 {
      Ok(())
    } else {
      Err(get_error())
    }
  }

  /// Blits the texture to the rendering target.
  ///
  /// * `src`: Optional clip rect of where to copy _from_. If None, the whole
  ///   texture is used.
  /// * `dst`: Optional clip rect of where to copy data _to_. If None, the whole
  ///   render target is used.
  ///
  /// The image is stretched as necessary if the `src` and `dst` are different
  /// sizes. This is a GPU operation, so it's fast no matter how much upscale or
  /// downscale you do.
  pub fn copy(&self, t: &Texture, src: Option<Rect>, dst: Option<Rect>) -> Result<(), String> {
    unsafe {
      let src_ptr = core::mem::transmute::<Option<&Rect>, *const SDL_Rect>(src.as_ref());
      let dst_ptr = core::mem::transmute::<Option<&Rect>, *const SDL_Rect>(dst.as_ref());
      if SDL_RenderCopy(self.ptr, t.ptr, src_ptr, dst_ptr) == 0 {
        Ok(())
      } else {
        Err(get_error())
      }
    }
  }

  /// Presents the backbuffer to the user.
  ///
  /// After a present, all backbuffer data should be assumed to be invalid, and
  /// you should also clear the backbuffer before doing the next render pass
  /// even if you intend to write to every pixel.
  pub fn present(&self) {
    unsafe { SDL_RenderPresent(self.ptr) };
  }
}
