use cgmath;

#[derive(Debug)]
pub struct PerspectiveCamera {
    eye: cgmath::Point3<f32>,
    center: cgmath::Point3<f32>,
    up: cgmath::Vector3<f32>,
    fovy: cgmath::Deg<f32>,
    aspect: f32,
    near: f32,
    far: f32,
    view: cgmath::Matrix4<f32>,
    proj: cgmath::Matrix4<f32>,
}

impl PerspectiveCamera {
    const OPENGL_TO_WEBGPU: cgmath::Matrix4<f32> = cgmath::Matrix4::new(
        1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.5, 0.5, 0.0, 0.0, 0.0, 1.0,
    );

    pub fn new() -> Self {
        Default::default()
    }

    pub fn to_view_proj(&self) -> [[f32; 4]; 4] {
        (Self::OPENGL_TO_WEBGPU * self.proj * self.view).into()
    }

    pub fn set_view(
        &mut self,
        eye: Option<(f32, f32, f32)>,
        center: Option<(f32, f32, f32)>,
        up: Option<(f32, f32, f32)>,
    ) {
        match eye {
            Some(x) => self.eye = x.into(),
            None => (),
        }
        match center {
            Some(x) => self.center = x.into(),
            None => (),
        }
        match up {
            Some(x) => self.up = x.into(),
            None => (),
        }
        self.view = cgmath::Matrix4::look_at_rh(self.eye, self.center, self.up);
    }

    pub fn set_proj(
        &mut self,
        fovy: Option<f32>,
        aspect: Option<f32>,
        near: Option<f32>,
        far: Option<f32>,
    ) {
        match fovy {
            Some(x) => self.fovy = cgmath::Deg(x),
            None => (),
        }
        match aspect {
            Some(x) => self.aspect = x,
            None => (),
        }
        match near {
            Some(x) => self.near = x,
            None => (),
        }
        match far {
            Some(x) => self.far = x,
            None => (),
        }
        self.proj = cgmath::perspective(self.fovy, self.aspect, self.near, self.far);
    }
}

impl Default for PerspectiveCamera {
    fn default() -> Self {
        let eye = (0.0, 0.0, 0.0).into();
        let center = (0.0, 0.0, 0.0).into();
        let up = (0.0, 1.0, 0.0).into();
        let fovy = cgmath::Deg(90.0);
        let aspect = 1.0;
        let near = 0.1;
        let far = 1000.0;
        let view = cgmath::Matrix4::look_at_rh(eye, center, up);
        let proj = cgmath::perspective(fovy, aspect, near, far);

        Self {
            eye,
            center,
            up,
            fovy,
            aspect,
            near,
            far,
            view,
            proj,
        }
    }
}
