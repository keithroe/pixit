use glam::IVec3;
use glam::Vec3;

pub struct BoundingBox {
    pub min: glam::Vec3,
    pub max: glam::Vec3,
}

impl BoundingBox {
    pub fn new(p0: glam::Vec3, p1: glam::Vec3) -> Self {
        let mut bbox = Self::default();
        bbox.expand_by_point(p0);
        bbox.expand_by_point(p1);
        bbox
    }

    pub fn expand_by_point(&mut self, p: glam::Vec3) {
        self.min = self.min.min(p);
        self.max = self.max.max(p);
    }

    pub fn expand_by_bbox(&mut self, bbox: &BoundingBox) {
        self.min = self.min.min(bbox.min);
        self.max = self.max.max(bbox.max);
    }
}

impl Default for BoundingBox {
    fn default() -> Self {
        Self {
            min: glam::Vec3::MAX,
            max: glam::Vec3::MIN,
        }
    }
}

#[derive(Default)]
pub struct Model {
    pub bbox: BoundingBox,
    pub verts: Vec<Vec3>,
    pub indices: Vec<IVec3>,
}

impl Model {
    pub fn load(gltf_file: &str) -> Self {
        let (document, buffers, _images) = gltf::import(gltf_file).expect("Failed to load GLTF");

        let mut model = Model::default();

        for node in document.nodes() {
            if let Some(mesh) = node.mesh() {
                println!("Found mesh '{}'", mesh.name().unwrap_or("<UNNAMED>"));
                println!("\txform: {:?}", node.transform());
                println!("\tprim count: {}", mesh.primitives().len());
                model.process_mesh(mesh, &buffers);
            }
        }
        model
    }

    fn process_mesh(&mut self, mesh: gltf::Mesh<'_>, buffers: &[gltf::buffer::Data]) {
        for primitive in mesh.primitives() {
            let bbox_gltf = primitive.bounding_box();
            let bbox = BoundingBox::new(
                glam::Vec3::from_slice(&bbox_gltf.min),
                glam::Vec3::from_slice(&bbox_gltf.max),
            );
            self.bbox.expand_by_bbox(&bbox);
            println!("\tprim bbox: {:?} - {:?}", bbox.min, bbox.max);

            let reader = primitive.reader(|buffer| Some(&buffers[buffer.index()]));
            if let Some(p) = reader.read_positions() {
                println!("\tP len: {}", p.len());
                self.verts = p.map(|x| Vec3::new(x[0], x[1], x[2])).collect();
                // TODO: transforms
            }

            if let Some(n) = reader.read_normals() {
                println!("\tN len: {}", n.len());
            } else {
                println!("\tN not found");
            }
            if let Some(cenum) = reader.read_colors(0) {
                match cenum {
                    gltf::mesh::util::ReadColors::RgbU8(i) => {
                        println!("RgbU8 len: {}", i.len());
                    }
                    gltf::mesh::util::ReadColors::RgbU16(i) => {
                        println!("RgbU16 len: {}", i.len());
                    }
                    gltf::mesh::util::ReadColors::RgbF32(i) => {
                        println!("RgbF32 len: {}", i.len());
                    }
                    gltf::mesh::util::ReadColors::RgbaU8(i) => {
                        println!("RgbaU8 len: {}", i.len());
                    }
                    gltf::mesh::util::ReadColors::RgbaU16(i) => {
                        println!("RgbaU16 len: {}", i.len());
                    }
                    gltf::mesh::util::ReadColors::RgbaF32(i) => {
                        println!("RgbaF32 len: {}", i.len());
                    }
                }
            } else {
                println!("\tC not found");
            }

            if let Some(ienum) = reader.read_indices() {
                match ienum {
                    gltf::mesh::util::ReadIndices::U8(i) => {
                        println!("I U8 len: {}", i.len());
                    }
                    gltf::mesh::util::ReadIndices::U16(i) => {
                        println!("I U16 len: {}", i.len());
                    }
                    gltf::mesh::util::ReadIndices::U32(i) => {
                        println!("I U32 len: {}", i.len());
                    }
                }
            }
        }
    }

    /*
    fn process_mesh(&mut self, mesh: gltf::Mesh<'_>, buffers: &Vec<buffer::Data>) {
        for primitive in mesh.primitives {
            let reader = primitive.reader(|buffer| Some(buffer_data[buffer.index()].as_slice()));
        }
    }
    */
}
