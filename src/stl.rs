fn copy_to_buf(buf:&mut [u8], copy:&[u8], start:usize) {
    for (i, byte) in copy.iter().enumerate() {
        buf[start + i] = *byte;
    }
}

#[derive(Debug, Copy, Clone)]
pub struct Vertex {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}
impl Vertex {
    pub fn new(x:f32, y:f32, z:f32) -> Self {
        Self { x, y, z }
    }

    pub fn to_bytes(&self) -> [u8; 12] {
        let mut buf = [0; 12];
        copy_to_buf(&mut buf, &self.x.to_le_bytes(), 0);
        copy_to_buf(&mut buf, &self.y.to_le_bytes(), 4);
        copy_to_buf(&mut buf, &self.z.to_le_bytes(), 8);

        buf
    }
}
impl From<(f32, f32, f32)> for Vertex {
    fn from((x, y, z):(f32, f32, f32)) -> Self {
        Self { x, y, z }
    }
}

pub struct Triangle {
    normal: Vertex,
    v1: Vertex,
    v2: Vertex,
    v3: Vertex,
}
impl Triangle {
    pub fn new(normal:Vertex, v1:Vertex, v2:Vertex, v3:Vertex) -> Self {
        Self { normal, v1, v2, v3 }
    }

    pub fn from_vertices(v1:Vertex, v2:Vertex, v3:Vertex) -> Self {
        let (ax, ay, az) = (v1.x - v3.x, v1.y - v3.y, v1.z - v3.z);
        let (bx, by, bz) = (v2.x - v3.x, v2.y - v3.y, v2.z - v3.z);
        let normal:Vertex = Vertex::new(ay * bz - az * by, az * bx - ax * bz, ax * by - ay * bx);

        Self::new(normal, v1, v2, v3)
    }

    pub fn from_2d_points(normal:Vertex, points:&[(f32, f32); 3], z:f32) -> Self {
        let vertices = points.map(|(x, y)| Vertex::new(x, y, z));
        Self::new(normal, vertices[0], vertices[1], vertices[2])
    }

    pub fn to_bytes(&self) -> [u8; 50] {
        let mut buf = [0; 50];
        copy_to_buf(&mut buf, &self.normal.to_bytes(), 0);
        copy_to_buf(&mut buf, &self.v1.to_bytes(), 12);
        copy_to_buf(&mut buf, &self.v2.to_bytes(), 24);
        copy_to_buf(&mut buf, &self.v3.to_bytes(), 36);

        buf
    }
}

pub fn generate_stl(triangles:&[Triangle]) -> Vec<u8> {
    let mut buf = vec![0; 80];
    buf.extend_from_slice(&(triangles.len() as u32).to_le_bytes());
    buf.extend(triangles.iter()
        .map(|triangle| triangle.to_bytes())
        .flatten());

    buf
}