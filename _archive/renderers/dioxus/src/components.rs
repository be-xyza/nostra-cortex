use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum ComponentWrapper {
    Text(TextProps),
    Image(ImageProps),
    Icon(IconProps),
    Video(VideoProps),
    AudioPlayer(AudioPlayerProps),
    Row(RowProps),
    Column(ColumnProps),
    List(ListProps),
    Card(CardProps),
    Tabs(TabsProps),
    Divider(DividerProps),
    Modal(ModalProps),
    Button(ButtonProps),
    CheckBox(CheckBoxProps),
    TextField(TextFieldProps),
    DateTimeInput(DateTimeInputProps),
    MultipleChoice(MultipleChoiceProps),
    Slider(SliderProps),
}

// --- Primitives ---

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct TextProps {
    pub text: PropertyValue<String>,
    #[serde(default)]
    pub usage_hint: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ImageProps {
    pub url: PropertyValue<String>,
    #[serde(default)]
    pub alt_text: Option<PropertyValue<String>>,
    #[serde(default)]
    pub fit: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct IconProps {
    pub name: PropertyValue<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct VideoProps {
    pub url: PropertyValue<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct AudioPlayerProps {
    pub url: PropertyValue<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct DividerProps {
    #[serde(default)]
    pub axis: Option<String>,
}

// --- Layouts ---

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct RowProps {
    pub children: ChildrenDefinition,
    #[serde(default)]
    pub distribution: Option<String>,
    #[serde(default)]
    pub alignment: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ColumnProps {
    pub children: ChildrenDefinition,
    #[serde(default)]
    pub distribution: Option<String>,
    #[serde(default)]
    pub alignment: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ListProps {
    pub children: ChildrenDefinition,
    #[serde(default)]
    pub direction: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct CardProps {
    pub child: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct TabsProps {
    pub tab_items: Vec<TabItem>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct TabItem {
    pub title: PropertyValue<String>,
    pub child: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ModalProps {
    pub entry_point_child: String,
    pub content_child: String,
}

// --- Inputs ---

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ButtonProps {
    pub child: String,
    pub action: ActionDefinition,
    #[serde(default)]
    pub primary: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct CheckBoxProps {
    pub label: PropertyValue<String>,
    pub value: PropertyValue<bool>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct TextFieldProps {
    pub label: PropertyValue<String>,
    #[serde(default)]
    pub text: Option<PropertyValue<String>>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct DateTimeInputProps {
    pub value: PropertyValue<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct MultipleChoiceProps {
    pub selections: PropertyValue<Vec<String>>,
    pub options: Vec<ChoiceOption>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ChoiceOption {
    pub label: PropertyValue<String>,
    pub value: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct SliderProps {
    pub value: PropertyValue<f64>,
}

// --- Helpers ---

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum ChildrenDefinition {
    ExplicitList(Vec<String>),
    Template(TemplateDefinition),
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct TemplateDefinition {
    pub component_id: String,
    pub data_binding: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ActionDefinition {
    pub name: String,
    #[serde(default)]
    pub context: Option<Vec<ContextEntry>>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct ContextEntry {
    pub key: String,
    pub value: PropertyValue<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(untagged)]
pub enum PropertyValue<T> {
    Literal(LiteralWrapper<T>),
    Path(PathWrapper),
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct LiteralWrapper<T> {
    #[serde(
        alias = "literalString",
        alias = "literalNumber",
        alias = "literalBoolean",
        alias = "literalArray"
    )]
    pub value: T,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct PathWrapper {
    pub path: String,
}
