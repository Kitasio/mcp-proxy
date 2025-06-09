use lib::server::Server;
use lib::types::{
    Tool, ToolImplementation, ToolsCallResult, error_content, success_content, text_content,
};
use serde_json::json;
use std::error::Error;

struct GreetTool;

impl ToolImplementation for GreetTool {
    fn get_tool(&self) -> Tool {
        Tool {
            name: "greet".to_string(),
            description: "Generate a greeting message for a given name".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "name": {
                        "type": "string",
                        "description": "The name to greet"
                    }
                },
                "required": ["name"]
            }),
        }
    }

    fn call(&self, arguments: serde_json::Value) -> ToolsCallResult {
        match arguments.get("name").and_then(|v| v.as_str()) {
            Some(name) => {
                let greeting = format!("Hello, {}! Welcome to the MCP server.", name);
                success_content(vec![text_content(greeting)])
            }
            None => error_content("Missing required parameter 'name'".to_string()),
        }
    }
}

struct GetTimeTool;

impl ToolImplementation for GetTimeTool {
    fn get_tool(&self) -> Tool {
        Tool {
            name: "get_time".to_string(),
            description: "Get the current system time".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {},
                "additionalProperties": false
            }),
        }
    }

    fn call(&self, _arguments: serde_json::Value) -> ToolsCallResult {
        use std::time::{SystemTime, UNIX_EPOCH};

        match SystemTime::now().duration_since(UNIX_EPOCH) {
            Ok(duration) => {
                let timestamp = duration.as_secs();
                let time_str = format!("Current Unix timestamp: {}", timestamp);
                success_content(vec![text_content(time_str)])
            }
            Err(_) => error_content("Failed to get system time".to_string()),
        }
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut server = Server::new();

    // Register tools
    server.register_tool(Box::new(GreetTool));
    server.register_tool(Box::new(GetTimeTool));

    server.run()?;

    Ok(())
}
