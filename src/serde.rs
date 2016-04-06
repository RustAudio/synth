extern crate serde;

mod envelope {
    use envelope::Envelope;
    use super::serde;

    impl serde::Serialize for Envelope {
        fn serialize<S>(&self, serializer: &mut S) -> Result<(), S::Error>
            where S: serde::Serializer,
        {
            struct Visitor<'a> {
                t: &'a Envelope,
                field_idx: u8,
            }

            impl<'a> serde::ser::MapVisitor for Visitor<'a> {
                fn visit<S>(&mut self, serializer: &mut S) -> Result<Option<()>, S::Error>
                    where S: serde::Serializer,
                {
                    match self.field_idx {
                        0 => {
                            self.field_idx += 1;
                            Ok(Some(try!(serializer.serialize_struct_elt("points",
                                                                         &self.t.points))))
                        },
                        _ => Ok(None),
                    }
                }

                fn len(&self) -> Option<usize> {
                    Some(1)
                }
            }

            serializer.serialize_struct("Envelope", Visitor { t: self, field_idx: 0 })
        }
    }

    impl serde::Deserialize for Envelope {
        fn deserialize<D>(deserializer: &mut D) -> Result<Self, D::Error>
            where D: serde::Deserializer,
        {
            struct Visitor;

            impl serde::de::Visitor for Visitor {
                type Value = Envelope;

                fn visit_map<V>(&mut self, mut visitor: V) -> Result<Envelope, V::Error>
                    where V: serde::de::MapVisitor,
                {
                    let mut points = None;

                    enum Field { Points }

                    impl serde::Deserialize for Field {
                        fn deserialize<D>(deserializer: &mut D) -> Result<Field, D::Error>
                            where D: serde::de::Deserializer,
                        {
                            struct FieldVisitor;

                            impl serde::de::Visitor for FieldVisitor {
                                type Value = Field;

                                fn visit_str<E>(&mut self, value: &str) -> Result<Field, E>
                                    where E: serde::de::Error,
                                {
                                    match value {
                                        "points" => Ok(Field::Points),
                                        _ => Err(serde::de::Error::custom("expected points")),
                                    }
                                }
                            }

                            deserializer.deserialize(FieldVisitor)
                        }
                    }

                    loop {
                        match try!(visitor.visit_key()) {
                            Some(Field::Points) => { points = Some(try!(visitor.visit_value())); },
                            None => { break; }
                        }
                    }

                    let points = match points {
                        Some(points) => points,
                        None => return Err(serde::de::Error::missing_field("points")),
                    };

                    try!(visitor.end());

                    Ok(Envelope { points: points })
                }
            }

            static FIELDS: &'static [&'static str] = &["hz", "amp"];

            deserializer.deserialize_struct("Envelope", FIELDS, Visitor)
        }
    }

    #[test]
    fn test() {
        use envelope::Point;
        extern crate serde_json;

        let envelope = Envelope { points: vec![Point { x: 0.5, y: 0.5, curve: 0.0 }] };
        let serialized = serde_json::to_string(&envelope).unwrap();

        println!("{}", serialized);
        assert_eq!("{\"points\":[{\"x\":0.5,\"y\":0.5,\"curve\":0}]}", serialized);
        
        let deserialized: Envelope = serde_json::from_str(&serialized).unwrap();

        println!("{:?}", deserialized);
        assert_eq!(envelope, deserialized);
    }
}

mod oscillator {

    mod waveform {

        mod sine {
            use oscillator::waveform::Sine;
            use super::super::super::serde;

            impl serde::Serialize for Sine {
                fn serialize<S>(&self, serializer: &mut S) -> Result<(), S::Error>
                    where S: serde::Serializer,
                {
                    serializer.serialize_unit_struct("Sine")
                }
            }

            impl serde::Deserialize for Sine {
                fn deserialize<D>(deserializer: &mut D) -> Result<Self, D::Error>
                    where D: serde::Deserializer,
                {
                    struct Visitor;

                    impl serde::de::Visitor for Visitor {
                        type Value = Sine;
    
                        fn visit_unit<E>(&mut self) -> Result<Self::Value, E>
                            where E: serde::de::Error,
                        {
                            Ok(Sine)
                        }
                    }

                    deserializer.deserialize_unit_struct("Sine", Visitor)
                }
            }

            #[test]
            fn test() {
                extern crate serde_json;

                let sine = Sine;
                let serialized = serde_json::to_string(&sine).unwrap();

                println!("{}", serialized);
                assert_eq!("null", &serialized);

                let deserialized: Sine = serde_json::from_str(&serialized).unwrap();

                println!("{:?}", deserialized);
                assert_eq!(sine, deserialized);
            }
        }

        mod saw {
            use oscillator::waveform::Saw;
            use super::super::super::serde;

            impl serde::Serialize for Saw {
                fn serialize<S>(&self, serializer: &mut S) -> Result<(), S::Error>
                    where S: serde::Serializer,
                {
                    serializer.serialize_unit_struct("Saw")
                }
            }

            impl serde::Deserialize for Saw {
                fn deserialize<D>(deserializer: &mut D) -> Result<Self, D::Error>
                    where D: serde::Deserializer,
                {
                    struct Visitor;

                    impl serde::de::Visitor for Visitor {
                        type Value = Saw;
    
                        fn visit_unit<E>(&mut self) -> Result<Self::Value, E>
                            where E: serde::de::Error,
                        {
                            Ok(Saw)
                        }
                    }

                    deserializer.deserialize_unit_struct("Saw", Visitor)
                }
            }

            #[test]
            fn test() {
                extern crate serde_json;

                let saw = Saw;
                let serialized = serde_json::to_string(&saw).unwrap();

                println!("{}", serialized);
                assert_eq!("null", &serialized);

                let deserialized: Saw = serde_json::from_str(&serialized).unwrap();

                println!("{:?}", deserialized);
                assert_eq!(saw, deserialized);
            }
        }

        mod saw_exp {
            use oscillator::waveform::SawExp;
            use super::super::super::serde;

            impl serde::Serialize for SawExp {
                fn serialize<S>(&self, serializer: &mut S) -> Result<(), S::Error>
                    where S: serde::Serializer,
                {
                    serializer.serialize_newtype_struct("SawExp", self.0)
                }
            }

            impl serde::Deserialize for SawExp {
                fn deserialize<D>(deserializer: &mut D) -> Result<Self, D::Error>
                    where D: serde::Deserializer,
                {
                    struct Visitor;

                    impl serde::de::Visitor for Visitor {
                        type Value = SawExp;

                        fn visit_f32<E>(&mut self, v: f32) -> Result<Self::Value, E>
                            where E: serde::de::Error,
                        {
                            Ok(SawExp(v))
                        }

                        fn visit_newtype_struct<D>(&mut self, deserializer: &mut D) -> Result<Self::Value, D::Error>
                            where D: serde::Deserializer,
                        {
                            Ok(SawExp(try!(serde::de::Deserialize::deserialize(deserializer))))
                        }
                    }

                    deserializer.deserialize_newtype_struct("SawExp", Visitor)
                }
            }

            #[test]
            fn test() {
                extern crate serde_json;

                let saw_exp = SawExp(2.0);
                let serialized = serde_json::to_string(&saw_exp).unwrap();

                println!("{}", serialized);
                assert_eq!("2", &serialized);

                let deserialized: SawExp = serde_json::from_str(&serialized).unwrap();

                println!("{:?}", deserialized);
                assert_eq!(saw_exp, deserialized);
            }
        }

        mod square {
            use oscillator::waveform::Square;
            use super::super::super::serde;

            impl serde::Serialize for Square {
                fn serialize<S>(&self, serializer: &mut S) -> Result<(), S::Error>
                    where S: serde::Serializer,
                {
                    serializer.serialize_unit_struct("Square")
                }
            }

            impl serde::Deserialize for Square {
                fn deserialize<D>(deserializer: &mut D) -> Result<Self, D::Error>
                    where D: serde::Deserializer,
                {
                    struct Visitor;

                    impl serde::de::Visitor for Visitor {
                        type Value = Square;
    
                        fn visit_unit<E>(&mut self) -> Result<Self::Value, E>
                            where E: serde::de::Error,
                        {
                            Ok(Square)
                        }
                    }

                    deserializer.deserialize_unit_struct("Square", Visitor)
                }
            }

            #[test]
            fn test() {
                extern crate serde_json;

                let square = Square;
                let serialized = serde_json::to_string(&square).unwrap();

                println!("{}", serialized);
                assert_eq!("null", &serialized);

                let deserialized: Square = serde_json::from_str(&serialized).unwrap();

                println!("{:?}", deserialized);
                assert_eq!(square, deserialized);
            }
        }

        mod noise {
            use oscillator::waveform::Noise;
            use super::super::super::serde;

            impl serde::Serialize for Noise {
                fn serialize<S>(&self, serializer: &mut S) -> Result<(), S::Error>
                    where S: serde::Serializer,
                {
                    serializer.serialize_unit_struct("Noise")
                }
            }

            impl serde::Deserialize for Noise {
                fn deserialize<D>(deserializer: &mut D) -> Result<Self, D::Error>
                    where D: serde::Deserializer,
                {
                    struct Visitor;

                    impl serde::de::Visitor for Visitor {
                        type Value = Noise;
    
                        fn visit_unit<E>(&mut self) -> Result<Self::Value, E>
                            where E: serde::de::Error,
                        {
                            Ok(Noise)
                        }
                    }

                    deserializer.deserialize_unit_struct("Noise", Visitor)
                }
            }

            #[test]
            fn test() {
                extern crate serde_json;

                let noise = Noise;
                let serialized = serde_json::to_string(&noise).unwrap();

                println!("{}", serialized);
                assert_eq!("null", &serialized);

                let deserialized: Noise = serde_json::from_str(&serialized).unwrap();

                println!("{:?}", deserialized);
                assert_eq!(noise, deserialized);
            }
        }

        mod noise_walk {
            use oscillator::waveform::NoiseWalk;
            use super::super::super::serde;

            impl serde::Serialize for NoiseWalk {
                fn serialize<S>(&self, serializer: &mut S) -> Result<(), S::Error>
                    where S: serde::Serializer,
                {
                    serializer.serialize_unit_struct("NoiseWalk")
                }
            }

            impl serde::Deserialize for NoiseWalk {
                fn deserialize<D>(deserializer: &mut D) -> Result<Self, D::Error>
                    where D: serde::Deserializer,
                {
                    struct Visitor;

                    impl serde::de::Visitor for Visitor {
                        type Value = NoiseWalk;
    
                        fn visit_unit<E>(&mut self) -> Result<Self::Value, E>
                            where E: serde::de::Error,
                        {
                            Ok(NoiseWalk)
                        }
                    }

                    deserializer.deserialize_unit_struct("NoiseWalk", Visitor)
                }
            }

            #[test]
            fn test() {
                extern crate serde_json;

                let noise_walk = NoiseWalk;
                let serialized = serde_json::to_string(&noise_walk).unwrap();

                println!("{}", serialized);
                assert_eq!("null", &serialized);

                let deserialized: NoiseWalk = serde_json::from_str(&serialized).unwrap();

                println!("{:?}", deserialized);
                assert_eq!(noise_walk, deserialized);
            }
        }

        mod dynamic {
            use oscillator::waveform::Dynamic;
            use super::super::super::serde;

            impl serde::Serialize for Dynamic {
                fn serialize<S>(&self, serializer: &mut S) -> Result<(), S::Error>
                    where S: serde::Serializer,
                {
                    match *self {
                        Dynamic::Sine => serializer.serialize_unit_variant("Dynamic", 0, "Sine"),
                        Dynamic::Saw => serializer.serialize_unit_variant("Dynamic", 1, "Saw"),
                        Dynamic::Square => serializer.serialize_unit_variant("Dynamic", 2, "Square"),
                        Dynamic::Noise => serializer.serialize_unit_variant("Dynamic", 3, "Noise"),
                        Dynamic::NoiseWalk => serializer.serialize_unit_variant("Dynamic", 4, "NoiseWalk"),
                        Dynamic::SawExp(ref s) => serializer.serialize_newtype_variant("Dynamic", 5, "SawExp", s),
                    }
                }
            }

            impl serde::Deserialize for Dynamic {
                fn deserialize<D>(deserializer: &mut D) -> Result<Self, D::Error>
                    where D: serde::Deserializer,
                {
                    enum Variant { Sine, Saw, Square, Noise, NoiseWalk, SawExp }

                    impl serde::de::Deserialize for Variant {
                        fn deserialize<D>(deserializer: &mut D) -> Result<Variant, D::Error>
                            where D: serde::Deserializer,
                        {
                            struct VariantVisitor;

                            impl serde::de::Visitor for VariantVisitor {
                                type Value = Variant;

                                fn visit_str<E>(&mut self, value: &str) -> Result<Variant, E>
                                    where E: serde::de::Error,
                                {
                                    match value {
                                        "Sine" => Ok(Variant::Sine),
                                        "Saw" => Ok(Variant::Saw),
                                        "Square" => Ok(Variant::Square),
                                        "Noise" => Ok(Variant::Noise),
                                        "NoiseWalk" => Ok(Variant::NoiseWalk),
                                        "SawExp" => Ok(Variant::SawExp),
                                        _ => Err(serde::de::Error::unknown_field(value)),
                                    }
                                }
                            }

                            deserializer.deserialize(VariantVisitor)
                        }
                    }

                    struct Visitor;

                    impl serde::de::EnumVisitor for Visitor {
                        type Value = Dynamic;

                        fn visit<V>(&mut self, mut visitor: V) -> Result<Self::Value, V::Error>
                            where V: serde::de::VariantVisitor,
                        {
                            match try!(visitor.visit_variant()) {
                                Variant::Sine => {
                                    try!(visitor.visit_unit());
                                    Ok(Dynamic::Sine)
                                },
                                Variant::Saw => {
                                    try!(visitor.visit_unit());
                                    Ok(Dynamic::Saw)
                                },
                                Variant::Square => {
                                    try!(visitor.visit_unit());
                                    Ok(Dynamic::Square)
                                },
                                Variant::Noise => {
                                    try!(visitor.visit_unit());
                                    Ok(Dynamic::Noise)
                                },
                                Variant::NoiseWalk => {
                                    try!(visitor.visit_unit());
                                    Ok(Dynamic::NoiseWalk)
                                },
                                Variant::SawExp => {
                                    let steepness = try!(visitor.visit_newtype());
                                    Ok(Dynamic::SawExp(steepness))
                                },
                            }
                        }
                    }

                    const VARIANTS: &'static [&'static str] = &[
                        "Sine", "Saw", "Square", "Noise", "NoiseWalk", "SawExp"
                    ];

                    deserializer.deserialize_enum("Dynamic", VARIANTS, Visitor)
                }
            }

            #[test]
            fn test() {
                extern crate serde_json;

                let saw_exp = Dynamic::SawExp(2.0);
                let serialized = serde_json::to_string(&saw_exp).unwrap();

                println!("{}", serialized);
                assert_eq!("{\"SawExp\":2}", serialized);
                
                let deserialized: Dynamic = serde_json::from_str(&serialized).unwrap();

                println!("{:?}", deserialized);
                assert_eq!(saw_exp, deserialized);
            }
        }

    }

    mod freq_warp {

        mod gaussian {
            use oscillator::freq_warp::Gaussian;
            use super::super::super::serde;

            impl serde::Serialize for Gaussian {
                fn serialize<S>(&self, serializer: &mut S) -> Result<(), S::Error>
                    where S: serde::Serializer,
                {
                    serializer.serialize_newtype_struct("Gaussian", self.0)
                }
            }

            impl serde::Deserialize for Gaussian {
                fn deserialize<D>(deserializer: &mut D) -> Result<Self, D::Error>
                    where D: serde::Deserializer,
                {
                    struct Visitor;

                    impl serde::de::Visitor for Visitor {
                        type Value = Gaussian;

                        fn visit_f32<E>(&mut self, v: f32) -> Result<Self::Value, E>
                            where E: serde::de::Error,
                        {
                            Ok(Gaussian(v))
                        }

                        fn visit_newtype_struct<D>(&mut self, deserializer: &mut D) -> Result<Self::Value, D::Error>
                            where D: serde::Deserializer,
                        {
                            Ok(Gaussian(try!(serde::de::Deserialize::deserialize(deserializer))))
                        }
                    }

                    deserializer.deserialize_newtype_struct("Gaussian", Visitor)
                }
            }

            #[test]
            fn test() {
                extern crate serde_json;

                let gaussian = Gaussian(2.0);
                let serialized = serde_json::to_string(&gaussian).unwrap();

                println!("{}", serialized);
                assert_eq!("2", &serialized);

                let deserialized: Gaussian = serde_json::from_str(&serialized).unwrap();

                println!("{:?}", deserialized);
                assert_eq!(gaussian, deserialized);
            }
        }

        mod pitch_drift {
            use oscillator::freq_warp::PitchDrift;
            use super::super::super::serde;

            impl serde::Serialize for PitchDrift {
                fn serialize<S>(&self, serializer: &mut S) -> Result<(), S::Error>
                    where S: serde::Serializer,
                {
                    struct Visitor<'a> {
                        t: &'a PitchDrift,
                        field_idx: u8,
                    }

                    impl<'a> serde::ser::MapVisitor for Visitor<'a> {
                        fn visit<S>(&mut self, serializer: &mut S) -> Result<Option<()>, S::Error>
                            where S: serde::Serializer,
                        {
                            match self.field_idx {
                                0 => {
                                    self.field_idx += 1;
                                    Ok(Some(try!(serializer.serialize_struct_elt("hz", self.t.hz))))
                                },
                                1 => {
                                    self.field_idx += 1;
                                    Ok(Some(try!(serializer.serialize_struct_elt("amp", self.t.amp))))
                                },
                                _ => Ok(None),
                            }
                        }

                        fn len(&self) -> Option<usize> {
                            Some(2)
                        }
                    }

                    serializer.serialize_struct("PitchDrift", Visitor { t: self, field_idx: 0 })
                }
            }

            impl serde::Deserialize for PitchDrift {
                fn deserialize<D>(deserializer: &mut D) -> Result<Self, D::Error>
                    where D: serde::Deserializer,
                {
                    struct Visitor;

                    impl serde::de::Visitor for Visitor {
                        type Value = PitchDrift;

                        fn visit_map<V>(&mut self, mut visitor: V) -> Result<PitchDrift, V::Error>
                            where V: serde::de::MapVisitor,
                        {
                            let mut hz = None;
                            let mut amp = None;

                            enum Field { Hz, Amp }

                            impl serde::Deserialize for Field {
                                fn deserialize<D>(deserializer: &mut D) -> Result<Field, D::Error>
                                    where D: serde::de::Deserializer,
                                {
                                    struct FieldVisitor;

                                    impl serde::de::Visitor for FieldVisitor {
                                        type Value = Field;

                                        fn visit_str<E>(&mut self, value: &str) -> Result<Field, E>
                                            where E: serde::de::Error,
                                        {
                                            match value {
                                                "hz" => Ok(Field::Hz),
                                                "amp" => Ok(Field::Amp),
                                                _ => Err(serde::de::Error::custom("expected hz or amp")),
                                            }
                                        }
                                    }

                                    deserializer.deserialize(FieldVisitor)
                                }
                            }

                            loop {
                                match try!(visitor.visit_key()) {
                                    Some(Field::Hz) => { hz = Some(try!(visitor.visit_value())); },
                                    Some(Field::Amp) => { amp = Some(try!(visitor.visit_value())); },
                                    None => { break; }
                                }
                            }

                            let hz = match hz {
                                Some(hz) => hz,
                                None => return Err(serde::de::Error::missing_field("hz")),
                            };

                            let amp = match amp {
                                Some(amp) => amp,
                                None => return Err(serde::de::Error::missing_field("amp")),
                            };

                            try!(visitor.end());

                            Ok(PitchDrift {
                                hz: hz,
                                amp: amp
                            })
                        }
                    }

                    static FIELDS: &'static [&'static str] = &["hz", "amp"];

                    deserializer.deserialize_struct("PitchDrift", FIELDS, Visitor)
                }
            }

            #[test]
            fn test() {
                extern crate serde_json;

                let pitch_drift = PitchDrift {
                    hz: 440.0,
                    amp: 1.0,
                };
                let serialized = serde_json::to_string(&pitch_drift).unwrap();

                println!("{}", serialized);
                assert_eq!("{\"hz\":440,\"amp\":1}", serialized);
                
                let deserialized: PitchDrift = serde_json::from_str(&serialized).unwrap();

                println!("{:?}", deserialized);
                assert_eq!(pitch_drift, deserialized);
            }
        }

        mod dynamic {
            use super::super::super::serde;
            use oscillator::freq_warp::Dynamic;

            impl serde::Serialize for Dynamic {
                fn serialize<S>(&self, serializer: &mut S) -> Result<(), S::Error>
                    where S: serde::Serializer,
                {
                    match *self {
                        Dynamic::None => serializer.serialize_unit_variant("Dynamic", 0, "None"),
                        Dynamic::Gaussian(g) => serializer.serialize_newtype_variant("Dynamic", 1, "Gaussian", g),
                        Dynamic::PitchDrift(p) => serializer.serialize_newtype_variant("Dynamic", 2, "PitchDrift", p),
                    }
                }
            }

            impl serde::Deserialize for Dynamic {
                fn deserialize<D>(deserializer: &mut D) -> Result<Self, D::Error>
                    where D: serde::Deserializer,
                {
                    enum Variant { None, Gaussian, PitchDrift }

                    impl serde::de::Deserialize for Variant {
                        fn deserialize<D>(deserializer: &mut D) -> Result<Variant, D::Error>
                            where D: serde::Deserializer,
                        {
                            struct VariantVisitor;

                            impl serde::de::Visitor for VariantVisitor {
                                type Value = Variant;

                                fn visit_str<E>(&mut self, value: &str) -> Result<Variant, E>
                                    where E: serde::de::Error,
                                {
                                    match value {
                                        "None" => Ok(Variant::None),
                                        "Gaussian" => Ok(Variant::Gaussian),
                                        "PitchDrift" => Ok(Variant::PitchDrift),
                                        _ => Err(serde::de::Error::unknown_field(value)),
                                    }
                                }
                            }

                            deserializer.deserialize(VariantVisitor)
                        }
                    }

                    struct Visitor;

                    impl serde::de::EnumVisitor for Visitor {
                        type Value = Dynamic;

                        fn visit<V>(&mut self, mut visitor: V) -> Result<Self::Value, V::Error>
                            where V: serde::de::VariantVisitor,
                        {
                            match try!(visitor.visit_variant()) {
                                Variant::None => {
                                    try!(visitor.visit_unit());
                                    Ok(Dynamic::None)
                                },
                                Variant::Gaussian => {
                                    let gaussian = try!(visitor.visit_newtype());
                                    Ok(Dynamic::Gaussian(gaussian))
                                },
                                Variant::PitchDrift => {
                                    let drift = try!(visitor.visit_newtype());
                                    Ok(Dynamic::PitchDrift(drift))
                                },
                            }
                        }
                    }

                    const VARIANTS: &'static [&'static str] = &[
                        "None", "Gaussian", "PitchDrift"
                    ];

                    deserializer.deserialize_enum("Dynamic", VARIANTS, Visitor)
                }
            }

            #[test]
            fn test() {
                use oscillator::freq_warp::Gaussian;
                extern crate serde_json;

                let gaussian = Dynamic::Gaussian(Gaussian(2.0));
                let serialized = serde_json::to_string(&gaussian).unwrap();

                println!("{}", serialized);
                assert_eq!("{\"Gaussian\":2}", serialized);
                
                let deserialized: Dynamic = serde_json::from_str(&serialized).unwrap();

                println!("{:?}", deserialized);
                assert_eq!(gaussian, deserialized);
            }
        }

    }

    mod amplitude {

        mod dynamic {
            use super::super::super::serde;
            use oscillator::amplitude::Dynamic;

            impl serde::Serialize for Dynamic {
                fn serialize<S>(&self, serializer: &mut S) -> Result<(), S::Error>
                    where S: serde::Serializer,
                {
                    match *self {
                        Dynamic::Envelope(ref e) => serializer.serialize_newtype_variant("Dynamic", 0, "Envelope", e),
                        Dynamic::Constant(a) => serializer.serialize_newtype_variant("Dynamic", 1, "Constant", a),
                    }
                }
            }

            impl serde::Deserialize for Dynamic {
                fn deserialize<D>(deserializer: &mut D) -> Result<Self, D::Error>
                    where D: serde::Deserializer,
                {
                    enum Variant { Envelope, Constant }

                    impl serde::de::Deserialize for Variant {
                        fn deserialize<D>(deserializer: &mut D) -> Result<Variant, D::Error>
                            where D: serde::Deserializer,
                        {
                            struct VariantVisitor;

                            impl serde::de::Visitor for VariantVisitor {
                                type Value = Variant;

                                fn visit_str<E>(&mut self, value: &str) -> Result<Variant, E>
                                    where E: serde::de::Error,
                                {
                                    match value {
                                        "Envelope" => Ok(Variant::Envelope),
                                        "Constant" => Ok(Variant::Constant),
                                        _ => Err(serde::de::Error::unknown_field(value)),
                                    }
                                }
                            }

                            deserializer.deserialize(VariantVisitor)
                        }
                    }

                    struct Visitor;

                    impl serde::de::EnumVisitor for Visitor {
                        type Value = Dynamic;

                        fn visit<V>(&mut self, mut visitor: V) -> Result<Self::Value, V::Error>
                            where V: serde::de::VariantVisitor,
                        {
                            match try!(visitor.visit_variant()) {
                                Variant::Envelope => {
                                    let env = try!(visitor.visit_newtype());
                                    Ok(Dynamic::Envelope(env))
                                },
                                Variant::Constant => {
                                    let amp = try!(visitor.visit_newtype());
                                    Ok(Dynamic::Constant(amp))
                                },
                            }
                        }
                    }

                    const VARIANTS: &'static [&'static str] = &["Envelope", "Constant"];

                    deserializer.deserialize_enum("Dynamic", VARIANTS, Visitor)
                }
            }

            #[test]
            fn test() {
                extern crate serde_json;

                let amp = Dynamic::Constant(1.0);
                let serialized = serde_json::to_string(&amp).unwrap();

                println!("{}", serialized);
                assert_eq!("{\"Constant\":1}", serialized);
                
                let deserialized: Dynamic = serde_json::from_str(&serialized).unwrap();

                println!("{:?}", deserialized);
                assert_eq!(amp, deserialized);
            }
        }


    }

    mod frequency {

        mod dynamic {
            use super::super::super::serde;
            use oscillator::frequency::Dynamic;

            impl serde::Serialize for Dynamic {
                fn serialize<S>(&self, serializer: &mut S) -> Result<(), S::Error>
                    where S: serde::Serializer,
                {
                    match *self {
                        Dynamic::Envelope(ref e) => serializer.serialize_newtype_variant("Dynamic", 0, "Envelope", e),
                        Dynamic::Hz(h) => serializer.serialize_newtype_variant("Dynamic", 1, "Hz", h),
                    }
                }
            }

            impl serde::Deserialize for Dynamic {
                fn deserialize<D>(deserializer: &mut D) -> Result<Self, D::Error>
                    where D: serde::Deserializer,
                {
                    enum Variant { Envelope, Hz }

                    impl serde::de::Deserialize for Variant {
                        fn deserialize<D>(deserializer: &mut D) -> Result<Variant, D::Error>
                            where D: serde::Deserializer,
                        {
                            struct VariantVisitor;

                            impl serde::de::Visitor for VariantVisitor {
                                type Value = Variant;

                                fn visit_str<E>(&mut self, value: &str) -> Result<Variant, E>
                                    where E: serde::de::Error,
                                {
                                    match value {
                                        "Envelope" => Ok(Variant::Envelope),
                                        "Hz" => Ok(Variant::Hz),
                                        _ => Err(serde::de::Error::unknown_field(value)),
                                    }
                                }
                            }

                            deserializer.deserialize(VariantVisitor)
                        }
                    }

                    struct Visitor;

                    impl serde::de::EnumVisitor for Visitor {
                        type Value = Dynamic;

                        fn visit<V>(&mut self, mut visitor: V) -> Result<Self::Value, V::Error>
                            where V: serde::de::VariantVisitor,
                        {
                            match try!(visitor.visit_variant()) {
                                Variant::Envelope => {
                                    let env = try!(visitor.visit_newtype());
                                    Ok(Dynamic::Envelope(env))
                                },
                                Variant::Hz => {
                                    let hz = try!(visitor.visit_newtype());
                                    Ok(Dynamic::Hz(hz))
                                },
                            }
                        }
                    }

                    const VARIANTS: &'static [&'static str] = &["Envelope", "Hz"];

                    deserializer.deserialize_enum("Dynamic", VARIANTS, Visitor)
                }
            }

            #[test]
            fn test() {
                extern crate serde_json;

                let hz = Dynamic::Hz(440.0);
                let serialized = serde_json::to_string(&hz).unwrap();

                println!("{}", serialized);
                assert_eq!("{\"Hz\":440}", serialized);
                
                let deserialized: Dynamic = serde_json::from_str(&serialized).unwrap();

                println!("{:?}", deserialized);
                assert_eq!(hz, deserialized);
            }
        }

    }

    mod oscillator {

        mod state {
            use oscillator::State;
            use super::super::super::serde;

            impl serde::Serialize for State {
                fn serialize<S>(&self, serializer: &mut S) -> Result<(), S::Error>
                    where S: serde::Serializer,
                {
                    struct Visitor<'a> {
                        t: &'a State,
                        field_idx: u8,
                    }

                    impl<'a> serde::ser::MapVisitor for Visitor<'a> {
                        fn visit<S>(&mut self, serializer: &mut S) -> Result<Option<()>, S::Error>
                            where S: serde::Serializer,
                        {
                            match self.field_idx {
                                0 => {
                                    self.field_idx += 1;
                                    Ok(Some(try!(serializer.serialize_struct_elt("phase", self.t.phase))))
                                },
                                1 => {
                                    self.field_idx += 1;
                                    Ok(Some(try!(serializer.serialize_struct_elt("freq_warp_phase",
                                                                                 self.t.freq_warp_phase))))
                                },
                                _ => Ok(None),
                            }
                        }

                        fn len(&self) -> Option<usize> {
                            Some(2)
                        }
                    }

                    serializer.serialize_struct("State", Visitor { t: self, field_idx: 0 })
                }
            }

            impl serde::Deserialize for State {
                fn deserialize<D>(deserializer: &mut D) -> Result<Self, D::Error>
                    where D: serde::Deserializer,
                {
                    struct Visitor;

                    impl serde::de::Visitor for Visitor {
                        type Value = State;

                        fn visit_map<V>(&mut self, mut visitor: V) -> Result<State, V::Error>
                            where V: serde::de::MapVisitor,
                        {
                            let mut phase = None;
                            let mut freq_warp_phase = None;

                            enum Field { Phase, FreqWarpPhase }

                            impl serde::Deserialize for Field {
                                fn deserialize<D>(deserializer: &mut D) -> Result<Field, D::Error>
                                    where D: serde::de::Deserializer,
                                {
                                    struct FieldVisitor;

                                    impl serde::de::Visitor for FieldVisitor {
                                        type Value = Field;

                                        fn visit_str<E>(&mut self, value: &str) -> Result<Field, E>
                                            where E: serde::de::Error,
                                        {
                                            match value {
                                                "phase" => Ok(Field::Phase),
                                                "freq_warp_phase" => Ok(Field::FreqWarpPhase),
                                                _ => Err(serde::de::Error::custom("expected phase or freq_warp_phase")),
                                            }
                                        }
                                    }

                                    deserializer.deserialize(FieldVisitor)
                                }
                            }

                            loop {
                                match try!(visitor.visit_key()) {
                                    Some(Field::Phase) => { phase = Some(try!(visitor.visit_value())); },
                                    Some(Field::FreqWarpPhase) => { freq_warp_phase = Some(try!(visitor.visit_value())); },
                                    None => { break; }
                                }
                            }

                            let phase = match phase {
                                Some(phase) => phase,
                                None => return Err(serde::de::Error::missing_field("phase")),
                            };

                            let freq_warp_phase = match freq_warp_phase {
                                Some(freq_warp_phase) => freq_warp_phase,
                                None => return Err(serde::de::Error::missing_field("freq_warp_phase")),
                            };

                            try!(visitor.end());

                            Ok(State {
                                phase: phase,
                                freq_warp_phase: freq_warp_phase,
                            })
                        }
                    }

                    static FIELDS: &'static [&'static str] = &["phase", "freq_warp_phase"];

                    deserializer.deserialize_struct("State", FIELDS, Visitor)
                }
            }

            #[test]
            fn test() {
                extern crate serde_json;

                let state = State {
                    phase: 0.0,
                    freq_warp_phase: 0.0,
                };
                let serialized = serde_json::to_string(&state).unwrap();

                println!("{}", serialized);
                assert_eq!("{\"phase\":0,\"freq_warp_phase\":0}", serialized);
                
                let deserialized: State = serde_json::from_str(&serialized).unwrap();

                println!("{:?}", deserialized);
                assert_eq!(state, deserialized);
            }
        }

        mod state_per_voice {
            use oscillator::StatePerVoice;
            use super::super::super::serde;

            impl serde::Serialize for StatePerVoice {
                fn serialize<S>(&self, serializer: &mut S) -> Result<(), S::Error>
                    where S: serde::Serializer,
                {
                    serializer.serialize_newtype_struct("StatePerVoice", &self.0)
                }
            }

            impl serde::Deserialize for StatePerVoice {
                fn deserialize<D>(deserializer: &mut D) -> Result<Self, D::Error>
                    where D: serde::Deserializer,
                {
                    struct Visitor;

                    impl serde::de::Visitor for Visitor {
                        type Value = StatePerVoice;

                        fn visit_newtype_struct<D>(&mut self, deserializer: &mut D) -> Result<Self::Value, D::Error>
                            where D: serde::Deserializer,
                        {
                            Ok(StatePerVoice(try!(serde::de::Deserialize::deserialize(deserializer))))
                        }
                    }

                    deserializer.deserialize_newtype_struct("StatePerVoice", Visitor)
                }
            }

            #[test]
            fn test() {
                extern crate serde_json;

                let state_per_voice = StatePerVoice(vec![]);
                let serialized = serde_json::to_string(&state_per_voice).unwrap();

                println!("{}", serialized);
                assert_eq!("[]", &serialized);

                let deserialized: StatePerVoice = serde_json::from_str(&serialized).unwrap();

                println!("{:?}", deserialized);
                assert_eq!(state_per_voice, deserialized);
            }
        }

        mod oscillator {
            use oscillator::Oscillator;
            use super::super::super::serde;
            use std;

            impl<W, A, F, FW> serde::Serialize for Oscillator<W, A, F, FW>
                where W: serde::Serialize,
                      A: serde::Serialize,
                      F: serde::Serialize,
                      FW: serde::Serialize,
            {
                fn serialize<S>(&self, serializer: &mut S) -> Result<(), S::Error>
                    where S: serde::Serializer,
                {
                    struct Visitor<'a, W: 'a, A: 'a, F: 'a, FW: 'a> {
                        t: &'a Oscillator<W, A, F, FW>,
                        field_idx: u8,
                    }

                    impl<'a, W, A, F, FW> serde::ser::MapVisitor for Visitor<'a, W, A, F, FW>
                        where W: serde::Serialize,
                              A: serde::Serialize,
                              F: serde::Serialize,
                              FW: serde::Serialize,
                    {
                        fn visit<S>(&mut self, serializer: &mut S) -> Result<Option<()>, S::Error>
                            where S: serde::Serializer,
                        {
                            match self.field_idx {
                                0 => {
                                    self.field_idx += 1;
                                    Ok(Some(try!(serializer.serialize_struct_elt("waveform",
                                                                                 &self.t.waveform))))
                                },
                                1 => {
                                    self.field_idx += 1;
                                    Ok(Some(try!(serializer.serialize_struct_elt("amplitude",
                                                                                 &self.t.amplitude))))
                                },
                                2 => {
                                    self.field_idx += 1;
                                    Ok(Some(try!(serializer.serialize_struct_elt("frequency",
                                                                                 &self.t.frequency))))
                                },
                                3 => {
                                    self.field_idx += 1;
                                    Ok(Some(try!(serializer.serialize_struct_elt("freq_warp",
                                                                                 &self.t.freq_warp))))
                                },
                                4 => {
                                    self.field_idx += 1;
                                    Ok(Some(try!(serializer.serialize_struct_elt("is_muted",
                                                                                 self.t.is_muted))))
                                },
                                _ => Ok(None),
                            }
                        }

                        fn len(&self) -> Option<usize> {
                            Some(5)
                        }
                    }

                    serializer.serialize_struct("Oscillator", Visitor { t: self, field_idx: 0 })
                }
            }

            impl<W, A, F, FW> serde::Deserialize for Oscillator<W, A, F, FW>
                where W: serde::Deserialize,
                      A: serde::Deserialize,
                      F: serde::Deserialize,
                      FW: serde::Deserialize,
            {
                fn deserialize<D>(deserializer: &mut D) -> Result<Self, D::Error>
                    where D: serde::Deserializer,
                {
                    struct Visitor<W, A, F, FW> {
                        w: std::marker::PhantomData<W>,
                        a: std::marker::PhantomData<A>,
                        f: std::marker::PhantomData<F>,
                        fw: std::marker::PhantomData<FW>,
                    }

                    impl<W, A, F, FW> serde::de::Visitor for Visitor<W, A, F, FW>
                        where W: serde::Deserialize,
                              A: serde::Deserialize,
                              F: serde::Deserialize,
                              FW: serde::Deserialize,
                    {
                        type Value = Oscillator<W, A, F, FW>;

                        fn visit_map<V>(&mut self, mut visitor: V) -> Result<Oscillator<W, A, F, FW>, V::Error>
                            where V: serde::de::MapVisitor,
                        {
                            let mut waveform = None;
                            let mut amplitude = None;
                            let mut frequency = None;
                            let mut freq_warp = None;
                            let mut is_muted = None;

                            enum Field {
                                Waveform,
                                Amplitude,
                                Frequency,
                                FreqWarp,
                                IsMuted,
                            }

                            impl serde::Deserialize for Field {
                                fn deserialize<D>(deserializer: &mut D) -> Result<Field, D::Error>
                                    where D: serde::de::Deserializer,
                                {
                                    struct FieldVisitor;

                                    impl serde::de::Visitor for FieldVisitor {
                                        type Value = Field;

                                        fn visit_str<E>(&mut self, value: &str) -> Result<Field, E>
                                            where E: serde::de::Error,
                                        {
                                            match value {
                                                "waveform" => Ok(Field::Waveform),
                                                "amplitude" => Ok(Field::Amplitude),
                                                "frequency" => Ok(Field::Frequency),
                                                "freq_warp" => Ok(Field::FreqWarp),
                                                "is_muted" => Ok(Field::IsMuted),
                                                _ => Err(serde::de::Error::custom(
                                                    "expected waveform, amplitude, frequency, \
                                                    start_mel or target_mel"
                                                )),
                                            }
                                        }
                                    }

                                    deserializer.deserialize(FieldVisitor)
                                }
                            }

                            loop {
                                match try!(visitor.visit_key()) {
                                    Some(Field::Waveform) => { waveform = Some(try!(visitor.visit_value())); },
                                    Some(Field::Amplitude) => { amplitude = Some(try!(visitor.visit_value())); },
                                    Some(Field::Frequency) => { frequency = Some(try!(visitor.visit_value())); },
                                    Some(Field::FreqWarp) => { freq_warp = Some(try!(visitor.visit_value())); },
                                    Some(Field::IsMuted) => { is_muted = Some(try!(visitor.visit_value())); },
                                    None => { break; }
                                }
                            }

                            let waveform = match waveform {
                                Some(waveform) => waveform,
                                None => return Err(serde::de::Error::missing_field("waveform")),
                            };

                            let amplitude = match amplitude {
                                Some(amplitude) => amplitude,
                                None => return Err(serde::de::Error::missing_field("amplitude")),
                            };

                            let frequency = match frequency {
                                Some(frequency) => frequency,
                                None => return Err(serde::de::Error::missing_field("frequency")),
                            };

                            let freq_warp = match freq_warp {
                                Some(freq_warp) => freq_warp,
                                None => return Err(serde::de::Error::missing_field("freq_warp")),
                            };

                            let is_muted = match is_muted {
                                Some(is_muted) => is_muted,
                                None => return Err(serde::de::Error::missing_field("is_muted")),
                            };

                            try!(visitor.end());

                            Ok(Oscillator {
                                waveform: waveform,
                                amplitude: amplitude,
                                frequency: frequency,
                                freq_warp: freq_warp,
                                is_muted: is_muted,
                            })
                        }
                    }

                    static FIELDS: &'static [&'static str] = &[
                        "waveform",
                        "amplitude",
                        "frequency",
                        "freq_warp",
                        "is_muted",
                    ];

                    deserializer.deserialize_struct("Oscillator", FIELDS, Visitor {
                        w: std::marker::PhantomData,
                        a: std::marker::PhantomData,
                        f: std::marker::PhantomData,
                        fw: std::marker::PhantomData,
                    })
                }
            }

            #[test]
            fn test() {
                use oscillator::waveform;

                extern crate serde_json;

                let osc = Oscillator::new(waveform::Sine, 1.0, 440.0, ());
                let serialized = serde_json::to_string(&osc).unwrap();

                println!("{}", serialized);
                assert_eq!("{\"waveform\":null,\"amplitude\":1,\"frequency\":440,\"freq_warp\":null,\"is_muted\":false}", serialized);
                
                let deserialized: Oscillator<waveform::Sine, f32, f64, ()> = serde_json::from_str(&serialized).unwrap();

                println!("{:?}", deserialized);
                assert_eq!(osc, deserialized);
            }
        }

    }

}

mod voice {
    use super::serde;
    use synth::Voice;

    impl serde::Serialize for Voice {
        fn serialize<S>(&self, serializer: &mut S) -> Result<(), S::Error>
            where S: serde::Serializer,
        {
            struct Visitor<'a> {
                t: &'a Voice,
                field_idx: u8,
            }

            impl<'a> serde::ser::MapVisitor for Visitor<'a> {
                fn visit<S>(&mut self, serializer: &mut S) -> Result<Option<()>, S::Error>
                    where S: serde::Serializer,
                {
                    match self.field_idx {
                        0 => {
                            self.field_idx += 1;
                            Ok(Some(try!(serializer.serialize_struct_elt("loop_playhead", self.t.loop_playhead))))
                        },
                        1 => {
                            self.field_idx += 1;
                            Ok(Some(try!(serializer.serialize_struct_elt("oscillator_states", &self.t.oscillator_states))))
                        },
                        _ => Ok(None),
                    }
                }

                fn len(&self) -> Option<usize> {
                    Some(2)
                }
            }

            serializer.serialize_struct("Voice", Visitor { t: self, field_idx: 0 })
        }
    }

    impl serde::Deserialize for Voice {
        fn deserialize<D>(deserializer: &mut D) -> Result<Self, D::Error>
            where D: serde::Deserializer,
        {
            struct Visitor;

            impl serde::de::Visitor for Visitor {
                type Value = Voice;

                fn visit_map<V>(&mut self, mut visitor: V) -> Result<Voice, V::Error>
                    where V: serde::de::MapVisitor,
                {
                    let mut loop_playhead = None;
                    let mut oscillator_states = None;

                    enum Field { LoopPlayhead, OscillatorStates }

                    impl serde::Deserialize for Field {
                        fn deserialize<D>(deserializer: &mut D) -> Result<Field, D::Error>
                            where D: serde::de::Deserializer,
                        {
                            struct FieldVisitor;

                            impl serde::de::Visitor for FieldVisitor {
                                type Value = Field;

                                fn visit_str<E>(&mut self, value: &str) -> Result<Field, E>
                                    where E: serde::de::Error,
                                {
                                    match value {
                                        "loop_playhead" => Ok(Field::LoopPlayhead),
                                        "oscillator_states" => Ok(Field::OscillatorStates),
                                        _ => Err(serde::de::Error::custom(
                                            "expected loop_playhead or oscillator_states"
                                        )),
                                    }
                                }
                            }

                            deserializer.deserialize(FieldVisitor)
                        }
                    }

                    loop {
                        match try!(visitor.visit_key()) {
                            Some(Field::LoopPlayhead) => { loop_playhead = Some(try!(visitor.visit_value())); },
                            Some(Field::OscillatorStates) => { oscillator_states = Some(try!(visitor.visit_value())); },
                            None => { break; }
                        }
                    }

                    let loop_playhead = match loop_playhead {
                        Some(loop_playhead) => loop_playhead,
                        None => return Err(serde::de::Error::missing_field("loop_playhead")),
                    };

                    let oscillator_states = match oscillator_states {
                        Some(oscillator_states) => oscillator_states,
                        None => return Err(serde::de::Error::missing_field("oscillator_states")),
                    };

                    try!(visitor.end());

                    Ok(Voice {
                        loop_playhead: loop_playhead,
                        oscillator_states: oscillator_states,
                    })
                }
            }

            static FIELDS: &'static [&'static str] = &["hz", "amp"];

            deserializer.deserialize_struct("Voice", FIELDS, Visitor)
        }
    }

    #[test]
    fn test() {
        use oscillator;
        extern crate serde_json;

        let voice = Voice {
            loop_playhead: 5,
            oscillator_states: oscillator::StatePerVoice(vec![]),
        };
        let serialized = serde_json::to_string(&voice).unwrap();

        println!("{}", serialized);
        assert_eq!("{\"loop_playhead\":5,\"oscillator_states\":[]}", serialized);
        
        let deserialized: Voice = serde_json::from_str(&serialized).unwrap();

        println!("{:?}", deserialized);
        assert_eq!(voice, deserialized);
    }
}

mod synth {
    use instrument::NoteFreqGenerator;
    use synth::Synth;
    use super::serde;
    use std;

    impl<M, NFG, W, A, F, FW> serde::Serialize for Synth<M, NFG, W, A, F, FW>
        where M: serde::Serialize,
              NFG: serde::Serialize + NoteFreqGenerator,
              NFG::NoteFreq: serde::Serialize,
              W: serde::Serialize,
              A: serde::Serialize,
              F: serde::Serialize,
              FW: serde::Serialize,
    {
        fn serialize<S>(&self, serializer: &mut S) -> Result<(), S::Error>
            where S: serde::Serializer,
        {
            struct Visitor<'a, M: 'a, NFG: 'a, W: 'a, A: 'a, F: 'a, FW: 'a>
                where NFG: NoteFreqGenerator,
            {
                t: &'a Synth<M, NFG, W, A, F, FW>,
                field_idx: u8,
            }

            impl<'a, M, NFG, W, A, F, FW> serde::ser::MapVisitor for Visitor<'a, M, NFG, W, A, F, FW>
                where M: serde::Serialize,
                      NFG: serde::Serialize + NoteFreqGenerator,
                      NFG::NoteFreq: serde::Serialize,
                      W: serde::Serialize,
                      A: serde::Serialize,
                      F: serde::Serialize,
                      FW: serde::Serialize,
            {
                fn visit<S>(&mut self, serializer: &mut S) -> Result<Option<()>, S::Error>
                    where S: serde::Serializer,
                {
                    match self.field_idx {
                        0 => {
                            self.field_idx += 1;
                            Ok(Some(try!(serializer.serialize_struct_elt("oscillators",
                                                                         &self.t.oscillators))))
                        },
                        1 => {
                            self.field_idx += 1;
                            Ok(Some(try!(serializer.serialize_struct_elt("voices",
                                                                         &self.t.voices))))
                        },
                        2 => {
                            self.field_idx += 1;
                            Ok(Some(try!(serializer.serialize_struct_elt("instrument",
                                                                         &self.t.instrument))))
                        },
                        3 => {
                            self.field_idx += 1;
                            Ok(Some(try!(serializer.serialize_struct_elt("volume",
                                                                         &self.t.volume))))
                        },
                        4 => {
                            self.field_idx += 1;
                            Ok(Some(try!(serializer.serialize_struct_elt("spread",
                                                                         &self.t.spread))))
                        },
                        5 => {
                            self.field_idx += 1;
                            Ok(Some(try!(serializer.serialize_struct_elt("loop_points",
                                                                         &self.t.loop_points))))
                        },
                        6 => {
                            self.field_idx += 1;
                            Ok(Some(try!(serializer.serialize_struct_elt("duration_ms",
                                                                         &self.t.duration_ms))))
                        },
                        7 => {
                            self.field_idx += 1;
                            Ok(Some(try!(serializer.serialize_struct_elt("base_pitch",
                                                                         &self.t.base_pitch))))
                        },
                        _ => Ok(None),
                    }
                }

                fn len(&self) -> Option<usize> {
                    Some(8)
                }
            }

            serializer.serialize_struct("Synth", Visitor { t: self, field_idx: 0 })
        }
    }

    impl<M, NFG, W, A, F, FW> serde::Deserialize for Synth<M, NFG, W, A, F, FW>
        where M: serde::Deserialize,
              NFG: serde::Deserialize + NoteFreqGenerator,
              NFG::NoteFreq: serde::Deserialize,
              W: serde::Deserialize,
              A: serde::Deserialize,
              F: serde::Deserialize,
              FW: serde::Deserialize,
    {
        fn deserialize<D>(deserializer: &mut D) -> Result<Self, D::Error>
            where D: serde::Deserializer,
        {
            struct Visitor<M, NFG, W, A, F, FW> {
                m: std::marker::PhantomData<M>,
                nfg: std::marker::PhantomData<NFG>,
                w: std::marker::PhantomData<W>,
                a: std::marker::PhantomData<A>,
                f: std::marker::PhantomData<F>,
                fw: std::marker::PhantomData<FW>,
            }

            impl<M, NFG, W, A, F, FW> serde::de::Visitor for Visitor<M, NFG, W, A, F, FW>
                where M: serde::Deserialize,
                      NFG: serde::Deserialize + NoteFreqGenerator,
                      NFG::NoteFreq: serde::Deserialize,
                      W: serde::Deserialize,
                      A: serde::Deserialize,
                      F: serde::Deserialize,
                      FW: serde::Deserialize,
            {
                type Value = Synth<M, NFG, W, A, F, FW>;

                fn visit_map<V>(&mut self, mut visitor: V) -> Result<Synth<M, NFG, W, A, F, FW>, V::Error>
                    where V: serde::de::MapVisitor,
                {
                    let mut oscillators = None;
                    let mut voices = None;
                    let mut instrument = None;
                    let mut volume = None;
                    let mut spread = None;
                    let mut loop_points = None;
                    let mut duration_ms = None;
                    let mut base_pitch = None;

                    enum Field {
                        Oscillators,
                        Voices,
                        Instrument,
                        Volume,
                        Spread,
                        LoopPoints,
                        DurationMs,
                        BasePitch,
                    }

                    impl serde::Deserialize for Field {
                        fn deserialize<D>(deserializer: &mut D) -> Result<Field, D::Error>
                            where D: serde::de::Deserializer,
                        {
                            struct FieldVisitor;

                            impl serde::de::Visitor for FieldVisitor {
                                type Value = Field;

                                fn visit_str<E>(&mut self, value: &str) -> Result<Field, E>
                                    where E: serde::de::Error,
                                {
                                    match value {
                                        "oscillators" => Ok(Field::Oscillators),
                                        "voices" => Ok(Field::Voices),
                                        "instrument" => Ok(Field::Instrument),
                                        "volume" => Ok(Field::Volume),
                                        "spread" => Ok(Field::Spread),
                                        "loop_points" => Ok(Field::LoopPoints),
                                        "duration_ms" => Ok(Field::DurationMs),
                                        "base_pitch" => Ok(Field::BasePitch),
                                        _ => Err(serde::de::Error::custom(
                                            "expected oscillators, voices, instrument, \
                                            volume, spread, loop_points, duration_ms or base_pitch"
                                        )),
                                    }
                                }
                            }

                            deserializer.deserialize(FieldVisitor)
                        }
                    }

                    loop {
                        match try!(visitor.visit_key()) {
                            Some(Field::Oscillators) => { oscillators = Some(try!(visitor.visit_value())); },
                            Some(Field::Voices) => { voices = Some(try!(visitor.visit_value())); },
                            Some(Field::Instrument) => { instrument = Some(try!(visitor.visit_value())); },
                            Some(Field::Volume) => { volume = Some(try!(visitor.visit_value())); },
                            Some(Field::Spread) => { spread = Some(try!(visitor.visit_value())); },
                            Some(Field::LoopPoints) => { loop_points = Some(try!(visitor.visit_value())); },
                            Some(Field::DurationMs) => { duration_ms = Some(try!(visitor.visit_value())); },
                            Some(Field::BasePitch) => { base_pitch = Some(try!(visitor.visit_value())); },
                            None => { break; }
                        }
                    }

                    let oscillators = match oscillators {
                        Some(oscillators) => oscillators,
                        None => return Err(serde::de::Error::missing_field("oscillators")),
                    };

                    let voices = match voices {
                        Some(voices) => voices,
                        None => return Err(serde::de::Error::missing_field("voices")),
                    };

                    let instrument = match instrument {
                        Some(instrument) => instrument,
                        None => return Err(serde::de::Error::missing_field("instrument")),
                    };

                    let volume = match volume {
                        Some(volume) => volume,
                        None => return Err(serde::de::Error::missing_field("volume")),
                    };

                    let spread = match spread {
                        Some(spread) => spread,
                        None => return Err(serde::de::Error::missing_field("spread")),
                    };

                    let loop_points = match loop_points {
                        Some(loop_points) => loop_points,
                        None => return Err(serde::de::Error::missing_field("loop_points")),
                    };

                    let duration_ms = match duration_ms {
                        Some(duration_ms) => duration_ms,
                        None => return Err(serde::de::Error::missing_field("duration_ms")),
                    };

                    let base_pitch = match base_pitch {
                        Some(base_pitch) => base_pitch,
                        None => return Err(serde::de::Error::missing_field("base_pitch")),
                    };

                    try!(visitor.end());

                    Ok(Synth {
                        oscillators: oscillators,
                        voices: voices,
                        instrument: instrument,
                        volume: volume,
                        spread: spread,
                        loop_points: loop_points,
                        duration_ms: duration_ms,
                        base_pitch: base_pitch,
                    })
                }
            }

            static FIELDS: &'static [&'static str] = &[
                "oscillators",
                "voices",
                "instrument",
                "volume",
                "spread",
                "loop_points",
                "duration_ms",
                "base_pitch",
            ];

            deserializer.deserialize_struct("Synth", FIELDS, Visitor {
                m: std::marker::PhantomData,
                nfg: std::marker::PhantomData,
                w: std::marker::PhantomData,
                a: std::marker::PhantomData,
                f: std::marker::PhantomData,
                fw: std::marker::PhantomData,
            })
        }
    }

    #[test]
    fn test() {
        use instrument::mode::Mono;
        use oscillator::{Oscillator, waveform};

        extern crate serde_json;

        let synth = Synth::legato(()).oscillator(Oscillator::new(waveform::Sine, 1.0, 440.0, ()));
        let serialized = serde_json::to_string(&synth).unwrap();

        println!("{}", serialized);
        
        let deserialized: Synth<Mono, (), waveform::Sine, f32, f64, ()> = serde_json::from_str(&serialized).unwrap();

        println!("{:?}", deserialized);
        assert_eq!(synth, deserialized);
    }

}
