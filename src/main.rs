use std::{fs, io, process};
use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use dotenv::dotenv;
use std::env;
use regex::Regex;

const DB_TYPES: &[&str] = &[
    "INTEGER", "BIGINT", "FLOAT", "REAL", "DOUBLE", "DECIMAL", "STRING", "TEXT",
    "BOOLEAN", "DATE", "DATEONLY", "TIME", "UUID", "JSON",
];

const DB_ATTR: &[&str] = &[
    "@PrimaryKey", "@AutoIncrement", "@Unique", "@Index",
    "@CreatedAt", "@UpdatedAt", "@DeletedAt", "@ForeignKey", "@BelongsTo",
    "@HasMany", "@HasOne", "@DefaultScope", "@Scopes", "@AllowNull",
    "@Comment", "@Default", "@Length", "@References",
];

const JS_TYPES: &[&str] = &[
    "number", "string", "boolean", "float", "double", "Date", "object",
    "function", "undefined", "symbol", "null"
];

#[derive(Debug, Clone)]
struct Field {
    attr: Vec<String>,
    name: String,
    db_type: String,
    js_type: String
}

impl Field {
    fn new(attr: Vec<&str>, name: &str, db_type: &str, js_type: &str) -> Self {
        Self {
            attr: attr.iter().map(|&value| value.to_string()).collect(),
            name: name.to_string(),
            db_type: db_type.to_string(),
            js_type: js_type.to_string(),
        }
    }

    fn validate(attr: Vec<&str>, name: &str, db_type: &str, js_type: &str) -> Result<Self, String> {
        if name.trim().is_empty() {
            return Err("Field name cannot be empty".to_string());
        }

        if !DB_TYPES.contains(&db_type) {
            return Err("Invalid database type".to_string());
        }

        if !JS_TYPES.contains(&js_type) {
            return Err("Invalid JavaScript type".to_string());
        }

        for attribute in &attr {
            if !DB_ATTR.contains(&attribute) {
                return Err(format!("Invalid attribute: {attribute}"));
            }
        }

        Ok(Self::new(attr, name, db_type, js_type))
    }
}

const NAME: &str = "Example_model_name";
const NAME_PLURAL: &str = "Example_model_name_plural";

fn copy_template(template_path: &str) -> io::Result<String>{
    let mut file = File::open(template_path)?;
    let mut content = String::new();
    file.read_to_string(&mut content)?;

    let content = content
        .replace("{NAME_UPPER}", NAME)
        .replace("{NAME_UPPER_PLURAL}", NAME_PLURAL)
        .replace("{NAME_LOWER}", &NAME.to_lowercase())
        .replace("{NAME_LOWER_PLURAL}", &NAME_PLURAL.to_lowercase());

    Ok(content)
}
fn implement_interface(path: PathBuf) -> io::Result<()>{
    let file_name = format!("I{NAME}Repository.ts");
    let file_path = path.join(&file_name);
    let mut file = File::create(&file_path)?;

    let template_path = env::var("INTERFACE_REPOSITORY_TEMPLATE").expect("INTERFACE_REPOSITORY_TEMPLATE not set in .env file");

    let content: String = copy_template(&template_path)?;

    file.write_all(content.as_bytes())?;

    Ok(())
}

fn implement_use_case(path: PathBuf, properties: Vec<Field>) -> io::Result<()>{
    let name_lower = NAME.to_lowercase();

    let new_path = path.join(NAME);

    fs::create_dir_all(&new_path).expect("Problem creating folder for use_case");

    let add_path = new_path.join(format!("Add{NAME}.ts"));
    let gets_path = new_path.join(format!("Get{NAME_PLURAL}.ts"));
    let delete_path = new_path.join(format!("Delete{NAME}.ts"));
    let update_path = new_path.join(format!("Update{NAME}.ts"));

    let mut add_file = File::create(&add_path)?;
    let mut gets_file = File::create(&gets_path).expect("File cannot be created");
    let mut delete_file = File::create(&delete_path)?;
    let mut update_file = File::create(&update_path)?;

    let add_template_path = env::var("ADD_USE_CASE_TEMPLATE").expect("ADD_USE_CASE_TEMPLATE not set in .env file");
    let gets_template_path = env::var("GETS_USE_CASE_TEMPLATE").expect("GETS_USE_CASE_TEMPLATE not set in .env file");
    let delete_template_path = env::var("DELETE_USE_CASE_TEMPLATE").expect("DELETE_USE_CASE_TEMPLATE not set in .env file");
    let update_template_path = env::var("UPDATE_USE_CASE_TEMPLATE").expect("UPDATE_USE_CASE_TEMPLATE not set in .env file");

    let mut add_content = copy_template(&add_template_path)?;
    let mut gets_content = copy_template(&gets_template_path)?;
    let mut delete_content = copy_template(&delete_template_path)?;
    let mut update_content = copy_template(&update_template_path)?;

    let mut dynamic_add_properties = String::new();
    let mut dynamic_update_properties = String::new();
    for (index, property) in properties.iter().enumerate() {
        if(property.name == "id"){
            continue;
        }
        if(dynamic_add_properties.len() > 0){
            dynamic_add_properties.push_str("\t\t\t");
        }
        if(dynamic_update_properties.len() > 0){
            dynamic_update_properties.push_str("\t\t");
        }
        dynamic_add_properties.push_str(&format!("{}: request.{},", property.name, property.name));
        dynamic_update_properties.push_str(&format!("{}.{} = request.{};", name_lower, property.name, property.name));
        if(index+1 != properties.len()){
            dynamic_add_properties.push_str("\n\n");
            dynamic_add_properties.push_str("\n");
        }
    }
    add_content = add_content.replace("{DYNAMIC_ADD_PROPERTIES}", &dynamic_add_properties);
    update_content = update_content.replace("{DYNAMIC_UPDATE_PROPERTIES}", &dynamic_update_properties);

    add_file.write_all(add_content.as_bytes()).expect("Error writing to add use case file");
    gets_file.write_all(gets_content.as_bytes()).expect("Error writing to gets use case file");
    delete_file.write_all(delete_content.as_bytes()).expect("Error writing to delete use case file");
    update_file.write_all(update_content.as_bytes()).expect("Error writing to update use case file");

    Ok(())
}

fn implement_utils(path: PathBuf, properties: Vec<Field>) -> io::Result<()>{
    let new_path = path.join(NAME);

    fs::create_dir_all(&new_path).expect("Problem creating folder for use_case");

    let mut request_file = File::create(new_path.join("Request.ts"))?;
    let mut types_file = File::create(new_path.join("types.ts"))?;

    let request_template_path = env::var("REQUEST_UTILS_TEMPLATE").expect("REPOSITORY_TEMPLATE not set in .env file");
    let types_template_path = env::var("TYPES_UTILS_TEMPLATE").expect("REPOSITORY_TEMPLATE not set in .env file");

    let mut request_content = copy_template(&request_template_path)?;
    let mut types_content = copy_template(&types_template_path)?;

    let mut dynamic_properties_attributes = String::new();
    let mut dynamic_properties_details = String::new();

    for (index, property) in properties.iter().enumerate() {
        if(dynamic_properties_attributes.len() > 0){
            dynamic_properties_attributes.push_str("\t");
        }
        if(dynamic_properties_details.len() > 0){
            dynamic_properties_details.push_str("\t");
        }

        dynamic_properties_attributes.push_str(&format!("{}: {};", property.name, property.js_type));
        if(property.name != "id"){
            dynamic_properties_details.push_str(&format!("{}: {};", property.name, property.js_type));
        }

        if(index+1 != properties.len()){
            dynamic_properties_attributes.push_str("\n");
            if dynamic_properties_details.len() > 0 {
                dynamic_properties_details.push_str("\n");
            };
        }
    }

    types_content = types_content.replace("{DYNAMIC_PROPERTIES_ATTRIBUTES}", &dynamic_properties_attributes);
    types_content = types_content.replace("{DYNAMIC_PROPERTIES_DETAILS}", &dynamic_properties_details);

    request_file.write_all(request_content.as_bytes())?;
    types_file.write_all(types_content.as_bytes())?;

    Ok(())
}

fn implement_repository(path: PathBuf) -> io::Result<()>{
    let file_path = path.join(&format!("{}Repository.ts", NAME.to_lowercase()));
    let mut file = File::create(&file_path)?;

    let template_path = env::var("REPOSITORY_TEMPLATE").expect("REPOSITORY_TEMPLATE not set in .env file");

    let mut content = copy_template(&template_path)?;

    file.write_all(content.as_bytes()).expect("Error writing to the repository file.");

    Ok(())
}

// fn implement_controllers(path: PathBuf);
fn implement_model(path: PathBuf, properties: Vec<Field>) -> io::Result<()>{
    let name_lower = NAME.to_lowercase();
    let file_name = format!("{name_lower}Model.ts");
    let file_path = path.join(&file_name);
    let mut file = File::create(&file_path)?;

    let template_path = env::var("MODEL_TEMPLATE").expect("MODEL_TEMPLATE not set in .env file");

    let mut dynamic_properties = String::new();
    for (index, item) in properties.iter().enumerate() {
        for attribute in &item.attr {
            dynamic_properties.push_str(&format!("\t{}\n", attribute));
        }
        dynamic_properties.push_str(&format!(
            "\t@Column(DataType.{})\n\t{}!: {};",
            item.db_type.to_uppercase(),
            item.name,
            item.js_type
        ));
        if index + 1 < properties.len() {
            dynamic_properties.push_str("\n\n");
        }
    }

    let mut content: String = copy_template(&template_path)?;

    content = content.replace("{DYNAMIC_PROPERTIES}", &dynamic_properties);

    file.write_all(content.as_bytes()).expect("Error writing to the model file.");

    Ok(())
}

fn implement_routes(path: PathBuf) -> io::Result<()>{
    let file_name = format!("{}Routes.ts", NAME.to_lowercase());
    let file_path = path.join(&file_name);
    let mut file = File::create(&file_path)?;

    let template_path = env::var("ROUTES_TEMPLATE").expect("ROUTES_TEMPLATE not set in .env file");

    let content: String = copy_template(&template_path)?;

    file.write_all(content.as_bytes())?;

    Ok(())
}
fn implement_controllers(path: PathBuf, properties: Vec<Field>) -> io::Result<()>{
    let file_name = format!("{}Controllers.ts", NAME.to_lowercase());
    let file_path = path.join(&file_name);
    let mut file = File::create(&file_path)?;

    let template_path = env::var("CONTROLLERS_TEMPLATE").expect("CONTROLLERS_TEMPLATE not set in .env file");

    let mut content: String = copy_template(&template_path)?;

    let mut dynamic_properties_details = String::new();

    for (index, property) in properties.iter().enumerate() {

        if(dynamic_properties_details.len() > 0){
            dynamic_properties_details.push_str("\t\t\t\t");
        }

        if(property.name != "id"){
            dynamic_properties_details.push_str(&format!("{}: req.body.{},", property.name, property.name));
        }

        if(index+1 != properties.len()){
            if dynamic_properties_details.len() > 0 {
                dynamic_properties_details.push_str("\n");
            };
        }
    }

    content = content.replace("{DYNAMIC_PROPERTIES_DETAILS}", &dynamic_properties_details);

    file.write_all(content.as_bytes())?;

    Ok(())
}

fn update_sequelize(path: PathBuf) -> io::Result<()>{
    let sequelize_path = path.join("sequelize.ts");
    let mut file_content = String::new();
    {
        let mut file = OpenOptions::new().read(true).open(sequelize_path.clone())?;
        file.read_to_string(&mut file_content)?;
    }

    let import = format!("import {{ {} }} from \"@infrastructure/models/{}Model\";\n", NAME, NAME.to_lowercase());
    if !file_content.contains(&import) {
        file_content = import + &file_content;
    }

    // Add model to models array
    let models_regex = Regex::new(r"models:\s*\[\s*(.*?)\s*]").unwrap();
    if let Some(captures) = models_regex.captures(&file_content) {
        let models_content = captures.get(1).unwrap().as_str();
        if !models_content.contains(NAME) {
            let updated_models_content = if models_content.is_empty() {
                format!("models: [{}]", NAME)
            } else {
                format!("models: [{}]", models_content.split(", ").chain(std::iter::once(NAME)).collect::<Vec<_>>().join(", "))
            };
            file_content = models_regex.replace(&file_content, updated_models_content).into_owned();
        }
    }

    {
        let mut file = OpenOptions::new().write(true).truncate(true).open(sequelize_path)?;
        file.write_all(file_content.as_bytes())?;
    }

    Ok(())
}

fn main() -> io::Result<()> {
    dotenv().ok();

    let main = Path::new("C:/Users/erlan/Documents/Spark/Clean Architecture");

    let directories = vec![
        ("core",
            vec!["interfaces", "use_cases", "utils"]),
        ("presentation",
            vec!["controllers"]),
        ("infrastructure",
            vec!["config", "models", "repositories", "routes"]),
    ];

    let properties: Vec<Field> = vec![
        Field::validate(
            vec!["@PrimaryKey", "@AutoIncrement"],
            "id",
            "INTEGER",
            "number"
        ).unwrap_or_else(|error| {
            println!("Error in fields: {error}");
            process::exit(1);
        }),
        Field::validate(
            vec![],
            "content",
            "STRING",
            "string"
        ).unwrap_or_else(|error| {
            println!("Error in fields: {error}");
            process::exit(1);
        }),
        Field::validate(
            vec![],
            "name",
            "STRING",
            "string"
        ).unwrap_or_else(|error| {
            println!("Error in fields: {error}");
            process::exit(1);
        })
    ];

    for (dir, subdirs) in directories{

        for subdir in subdirs{
            let current_dir = main.join(dir).join(subdir);

            if !current_dir.exists() {
                fs::create_dir_all(&current_dir)?;
            }
            match current_dir.to_str() {
                Some(path_str) => println!("{}", path_str),
                None => println!("Failed to convert PathBuf to string"),
            }
            if subdir == "models"{
                implement_model(current_dir.clone(), properties.clone())?;
            }
            if subdir == "interfaces" {
                implement_interface(current_dir.clone())?;
            }
            if subdir == "utils" {
                implement_utils(current_dir.clone(), properties.clone())?;
            }
            if subdir == "use_cases" {
                implement_use_case(current_dir.clone(), properties.clone())?;
            }
            if subdir == "repositories" {
                implement_repository(current_dir.clone())?;
            }
            if subdir == "controllers" {
                implement_controllers(current_dir.clone(), properties.clone())?;
            }
            if subdir == "routes" {
                implement_routes(current_dir.clone())?;
            }
            if subdir == "config" {
                update_sequelize(current_dir.clone())?;
            }
        }
    }

    Ok(())
}
