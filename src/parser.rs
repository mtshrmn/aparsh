#![allow(dead_code)]
use indoc::indoc;
use minijinja::value::{StructObject, Value};
use minijinja::Environment;
use serde::Deserialize;
use toml::de::Error;

static SCRIPT_TEMPLATE: &str = indoc! {
    r##"
    #!/bin/sh

    __usage="{{name}} - {{description}}
    Usage: {{name}} {%- for arg in positional %} {{arg.name}} {%-endfor %} [OPTIONS]

    Options:
      {%- for flag in flags %}
      {% if flag.short %}-{{flag.short}}, {% endif %}--{{flag.name}} {{flag.varname}}{% for i in range(maxlen - flag.len + 5) %} {% endfor %}{{flag.description}}
      {%- endfor %}
      --help{% for i in range(maxlen) %} {% endfor %}show this message and exit.
      {%- if prologue %}

    {{ prologue }}
    {%- endif %}"

    if [ $# -lt {{ positional | count }} ]; then
      echo "Not enough arguments - expected {{ positional | count }}, recieved $#"
      exit 1
    fi

    # initialize positional arguments

    {%- for arg in positional %}
    {{arg.name}}="${{loop.index}}"
    {%- endfor %}
    shift {{ positional | count }}

    # set default values for flags
    {%- for flag in flags %}
    {{flag.varname}}=
    {%- if flag.type == "store_true" -%}
    false
    {%- else -%}
    "{{flag.default}}"
    {%- endif %}
    {%- endfor %}

    while [[ $# -gt 0 ]]; do
      case $1 in
      {%- for flag in flags %}
        {% if flag.short %}-{{flag.short}}|{% endif %}--{{flag.name}})
          {% if flag.type != "store_true" -%}
          if [ $# -eq 1 ]; then
            echo "no value provided for --{{flag.name}}"
            exit 1
          fi
          {{flag.varname}}=$2
          shift 2
          {%- else -%}
          {{flag.varname}}=true
          shift
          {%- endif %}
          ;;
      {% endfor %}
        --help)
          echo "$__usage"
          exit
          ;;
        *)
          echo -e "Unkown flag \"$1\"\nTry {{name}} --help for help."
          exit 1
          ;;
      esac
    done
    "##
};

#[derive(Debug, Deserialize, Clone)]
struct PositionalArg {
    name: String,
    description: Option<String>,
    varname: Option<String>,
    #[serde(rename = "type")]
    ftype: Option<String>,
    nargs: Option<String>,
}

impl StructObject for PositionalArg {
    fn get_field(&self, field: &str) -> Option<Value> {
        match field {
            "name" => Some(Value::from(self.name.to_uppercase())),
            _ => None,
        }
    }
}

impl From<PositionalArg> for Value {
    fn from(p: PositionalArg) -> Self {
        Value::from_struct_object(p)
    }
}

#[derive(Debug, Deserialize, Clone)]
struct Flag {
    name: String,
    short: Option<String>,
    description: Option<String>,
    varname: Option<String>,
    default: Option<String>,
    #[serde(rename = "type")]
    ftype: Option<String>,
    nargs: Option<String>,
    choice: Option<Vec<String>>,
}

impl Flag {
    // calculate the length of the following string:
    // "-short, --long VARNAME"
    fn len(&self) -> usize {
        let short_size = self.short.as_ref().map(|s| s.len() + 3).unwrap_or(0);
        let varname_size = self.varname.as_deref().unwrap_or(self.name.as_str()).len();
        short_size + "--".len() + self.name.len() + varname_size
    }
}

impl StructObject for Flag {
    fn get_field(&self, field: &str) -> Option<Value> {
        match field {
            "name" => Some(Value::from(self.name.as_str())),
            "short" => self.short.as_deref().map(Value::from),
            "varname" => Some(Value::from(
                self.varname
                    .as_deref()
                    .unwrap_or(self.name.as_str())
                    .to_uppercase(),
            )),
            "description" => self.description.as_deref().map(Value::from),
            "len" => Some(Value::from(self.len())),
            "default" => self.default.as_deref().map(Value::from),
            "type" => self.ftype.as_deref().map(Value::from),
            _ => None,
        }
    }
}

impl From<Flag> for Value {
    fn from(f: Flag) -> Self {
        Value::from_struct_object(f)
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    name: String,
    description: Option<String>,
    prologue: Option<String>,
    positional: Option<Vec<PositionalArg>>,
    #[serde(rename = "flag")]
    flags: Option<Vec<Flag>>,
}

impl StructObject for Config {
    fn get_field(&self, field: &str) -> Option<Value> {
        match field {
            "name" => Some(Value::from(self.name.as_str())),
            "description" => self.description.as_deref().map(Value::from),
            "flags" => self.flags.as_ref().map(|f| (Value::from_iter(f.clone()))),
            "positional" => self
                .positional
                .as_ref()
                .map(|p| (Value::from_iter(p.clone()))),
            "maxlen" => Some(Value::from(
                self.flags
                    .as_ref()
                    .map(|flags| flags.clone().iter().map(|f| f.len()).max().unwrap_or(6))
                    .unwrap_or(6),
            )),
            "prologue" => self.prologue.as_deref().map(Value::from),
            _ => None,
        }
    }
}

#[derive(Debug)]
pub struct FlagParser {
    config: Config,
}

impl FlagParser {
    pub fn new(content: &str) -> Result<Self, Error> {
        Ok(Self {
            config: toml::from_str(content)?,
        })
    }

    pub fn render(&self) -> String {
        let env = Environment::new();
        let template = env.template_from_str(SCRIPT_TEMPLATE).unwrap();
        let ctx = Value::from_struct_object(self.config.clone());
        template.render(ctx).unwrap()
    }
}
