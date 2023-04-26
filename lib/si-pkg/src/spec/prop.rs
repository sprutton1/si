use derive_builder::UninitializedFieldError;
use serde::{Deserialize, Serialize};

use super::{AttrFuncInputSpec, FuncUniqueId, SpecError, ValidationSpec};

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum PropSpec {
    #[serde(rename_all = "camelCase")]
    String {
        name: String,
        validations: Vec<ValidationSpec>,
        func_unique_id: Option<FuncUniqueId>,
        inputs: Vec<AttrFuncInputSpec>,
    },
    #[serde(rename_all = "camelCase")]
    Number {
        name: String,
        validations: Vec<ValidationSpec>,
        func_unique_id: Option<FuncUniqueId>,
        inputs: Vec<AttrFuncInputSpec>,
    },
    #[serde(rename_all = "camelCase")]
    Boolean {
        name: String,
        validations: Vec<ValidationSpec>,
        func_unique_id: Option<FuncUniqueId>,
        inputs: Vec<AttrFuncInputSpec>,
    },
    #[serde(rename_all = "camelCase")]
    Map {
        name: String,
        type_prop: Box<PropSpec>,
        validations: Vec<ValidationSpec>,
        func_unique_id: Option<FuncUniqueId>,
        inputs: Vec<AttrFuncInputSpec>,
    },
    #[serde(rename_all = "camelCase")]
    Array {
        name: String,
        type_prop: Box<PropSpec>,
        validations: Vec<ValidationSpec>,
        func_unique_id: Option<FuncUniqueId>,
        inputs: Vec<AttrFuncInputSpec>,
    },
    #[serde(rename_all = "camelCase")]
    Object {
        name: String,
        entries: Vec<PropSpec>,
        validations: Vec<ValidationSpec>,
        func_unique_id: Option<FuncUniqueId>,
        inputs: Vec<AttrFuncInputSpec>,
    },
}

impl PropSpec {
    pub fn builder() -> PropSpecBuilder {
        PropSpecBuilder::default()
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum PropSpecKind {
    String,
    Number,
    Boolean,
    Map,
    Array,
    Object,
}

#[derive(Clone, Debug, Default)]
pub struct PropSpecBuilder {
    kind: Option<PropSpecKind>,
    name: Option<String>,
    type_prop: Option<PropSpec>,
    entries: Vec<PropSpec>,
    validations: Vec<ValidationSpec>,
    func_unique_id: Option<FuncUniqueId>,
    inputs: Vec<AttrFuncInputSpec>,
}

impl PropSpecBuilder {
    #[allow(unused_mut)]
    pub fn kind(&mut self, value: PropSpecKind) -> &mut Self {
        self.kind = Some(value);
        self
    }

    pub fn get_kind(&self) -> Option<PropSpecKind> {
        self.kind
    }

    #[allow(unused_mut)]
    pub fn name(&mut self, value: impl Into<String>) -> &mut Self {
        self.name = Some(value.into());
        self
    }

    #[allow(unused_mut)]
    pub fn type_prop(&mut self, value: impl Into<PropSpec>) -> &mut Self {
        self.type_prop = Some(value.into());
        self
    }

    #[allow(unused_mut)]
    pub fn entry(&mut self, value: impl Into<PropSpec>) -> &mut Self {
        self.entries.push(value.into());
        self
    }

    #[allow(unused_mut)]
    pub fn validation(&mut self, value: impl Into<ValidationSpec>) -> &mut Self {
        self.validations.push(value.into());
        self
    }

    #[allow(unused_mut)]
    pub fn entries(&mut self, value: Vec<impl Into<PropSpec>>) -> &mut Self {
        self.entries = value.into_iter().map(Into::into).collect();
        self
    }

    #[allow(unused_mut)]
    pub fn func_unique_id(&mut self, value: FuncUniqueId) -> &mut Self {
        self.func_unique_id = Some(value);
        self
    }

    #[allow(unused_mut)]
    pub fn input(&mut self, value: impl Into<AttrFuncInputSpec>) -> &mut Self {
        self.inputs.push(value.into());
        self
    }

    /// Builds a new `Prop`.
    ///
    /// # Errors
    ///
    /// If a required field has not been initialized.
    pub fn build(&self) -> Result<PropSpec, SpecError> {
        let name = match self.name {
            Some(ref name) => name.clone(),
            None => {
                return Err(UninitializedFieldError::from("name").into());
            }
        };

        let validations = self.validations.clone();
        let inputs = self.inputs.clone();
        let func_unique_id = self.func_unique_id;

        Ok(match self.kind {
            Some(kind) => match kind {
                PropSpecKind::String => PropSpec::String {
                    name,
                    validations,
                    func_unique_id,
                    inputs,
                },
                PropSpecKind::Number => PropSpec::Number {
                    name,
                    validations,
                    func_unique_id,
                    inputs,
                },
                PropSpecKind::Boolean => PropSpec::Boolean {
                    name,
                    validations,
                    func_unique_id,
                    inputs,
                },
                PropSpecKind::Map => PropSpec::Map {
                    name,
                    type_prop: match self.type_prop {
                        Some(ref value) => Box::new(value.clone()),
                        None => {
                            return Err(UninitializedFieldError::from("type_prop").into());
                        }
                    },
                    validations,
                    func_unique_id,
                    inputs,
                },
                PropSpecKind::Array => PropSpec::Array {
                    name,
                    type_prop: match self.type_prop {
                        Some(ref value) => Box::new(value.clone()),
                        None => {
                            return Err(UninitializedFieldError::from("type_prop").into());
                        }
                    },
                    validations,
                    func_unique_id,
                    inputs,
                },
                PropSpecKind::Object => PropSpec::Object {
                    name,
                    entries: self.entries.clone(),
                    validations,
                    func_unique_id,
                    inputs,
                },
            },
            None => {
                return Err(UninitializedFieldError::from("kind").into());
            }
        })
    }
}

impl TryFrom<PropSpecBuilder> for PropSpec {
    type Error = SpecError;

    fn try_from(value: PropSpecBuilder) -> Result<Self, Self::Error> {
        value.build()
    }
}
