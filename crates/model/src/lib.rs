use glam::UVec3;
use glam::Vec3;

use itertools::Itertools;

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

    pub fn mid(&self) -> glam::Vec3 {
        (self.min + self.max) * 0.5
    }

    pub fn longest_axis(&self) -> f32 {
        (self.max - self.min).max_element()
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
pub struct Primitive {
    /// List of vertex index triples.
    ///
    /// Each represents a triangle's attribute index for each of its three vertices.  If
    /// `indices` is empty, all attributes are stored in a flat list:
    /// [ tri0_vert0_attr_val, tri0_vert1_attr_val, tri0_vert1_attr_val, tri1_vert0_attr_val, ...]
    pub indices: Vec<UVec3>,

    /// POSITION vertex attribute.  
    ///
    /// Object space 3D positions of the mesh vertices. Positions is the only required attribute.
    pub positions: Vec<Vec3>,

    /// TEXCOORD_<N> vertex attributes.
    ///
    /// 2D texture coordinates.  A single model may have multiple tex coord sets.
    pub texcoords: Vec<Vec<glam::Vec2>>,

    /// TANGENTS vertex attribute.
    ///
    /// XYZW tangents where the W component indicates handedness of the 3D tangent XYZ. (-1 or 1)
    pub tangents: Vec<glam::Vec4>,

    /// JOINTS_<N> vertex attributes.
    ///
    /// List of joints influencing each vertex.  This list is always length 4 (maximum number
    /// possible.  If there are fewer than 4 joints for a given vertex, the `weights` attribute
    /// will be set to zero for the inactive joint indices. A single model may have multiple
    /// joint/weight sets.
    pub joints: Vec<Vec<glam::U16Vec4>>,

    /// WEIGHTS_<N> vertex attributes.
    ///
    /// List of weights for the joints influencing each vertex.  This list is length 4, but some
    /// weights may be zero. A single model may have multiple weight/joint sets.
    pub weights: Vec<Vec<glam::Vec4>>,

    /// COLORS_<N> vertex attributes.
    ///
    /// List of vertex colors.  All colors upconverted to f32 RGBA (eg, from U8)
    pub colors: Vec<glam::Vec4>,
}

/// 3D triangle mesh model.
///
/// Based looselty on the GLTF model format.  Triangle may be indexed or a flat list of vertex
/// attributes. All attributes except for joint indices are converted up to f32 (eg, from u16). All
/// vertex attributes are copied into separate, tightly packed arrays (de-interleaved and
/// de-offset).
#[derive(Default)]
pub struct Mesh {
    /// Object space bounding box of the model
    pub bbox: BoundingBox,

    /// List of mesh primitives with pre-transformed vertex data
    pub primitives: Vec<Primitive>,
}

impl Mesh {
    /// Convert gltf model to our in-memory model format
    pub fn from_gltf(gltf_file: &str) -> Self {
        let mut mesh = Mesh::default();

        let (document, buffers, _images) = gltf::import(gltf_file).expect("Failed to load GLTF");
        for root_node in document.nodes() {
            mesh.process_node(&root_node, &buffers, glam::Mat4::IDENTITY);
        }

        mesh
    }

    /// Process the GLTF root node, traversing the node tree, accumulating
    /// transforms and creating pre-transformed meshes
    fn process_node(
        &mut self,
        node: &gltf::Node,
        buffers: &[gltf::buffer::Data],
        transform: glam::Mat4,
    ) {
        let node_transform = glam::Mat4::from_cols_slice(node.transform().matrix().as_flattened());
        let transform = node_transform * transform;
        if let Some(mesh) = node.mesh() {
            println!("Found mesh '{}'", mesh.name().unwrap_or("<UNNAMED>"));
            println!("\txform: {:?}", node.transform());
            println!("\tprim count: {}", mesh.primitives().len());
            self.process_mesh(mesh, &buffers, &transform);
        }

        for child in node.children() {
            self.process_node(&child, buffers, transform);
        }
    }

    fn process_mesh(
        &mut self,
        mesh: gltf::Mesh<'_>,
        buffers: &[gltf::buffer::Data],
        transform: &glam::Mat4,
    ) {
        for primitive in mesh.primitives() {
            for attr in primitive.attributes() {
                println!("\t\tattr: {}", attr.0.to_string());
            }

            let bbox_gltf = primitive.bounding_box();
            let bbox = BoundingBox::new(
                transform.transform_point3(glam::Vec3::from_slice(&bbox_gltf.min)),
                transform.transform_point3(glam::Vec3::from_slice(&bbox_gltf.max)),
            );
            self.bbox.expand_by_bbox(&bbox);
            println!("\txformed prim bbox: {:?} - {:?}", bbox.min, bbox.max);

            let mut prim = Primitive::default();

            let reader = primitive.reader(|buffer| Some(&buffers[buffer.index()]));
            if let Some(p) = reader.read_positions() {
                println!("\tP len: {}", p.len());
                prim.positions = p
                    .map(|x| transform.transform_point3(Vec3::new(x[0], x[1], x[2])))
                    .collect();
            }

            if let Some(ienum) = reader.read_indices() {
                prim.indices = ienum
                    .into_u32()
                    .tuples()
                    .map(|(x, y, z)| UVec3::new(x, y, z))
                    .collect();
                println!("\tI len: {}", prim.indices.len());
            } else {
                println!("\tI not found");
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

            if let Some(uvenum) = reader.read_tex_coords(0) {
                match uvenum {
                    gltf::mesh::util::ReadTexCoords::U8(i) => {
                        println!("UV U8 len: {}", i.len());
                    }
                    gltf::mesh::util::ReadTexCoords::U16(i) => {
                        println!("UV U16 len: {}", i.len());
                    }
                    gltf::mesh::util::ReadTexCoords::F32(i) => {
                        println!("UV F32 len: {}", i.len());
                    }
                }
            } else {
                println!("\tUV not found");
            }
            self.primitives.push(prim);
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
