# MCP App SDK Guide

## Overview
The MCP SDK allows developers to build "Agent-First" applications. Instead of focusing on UI, developers focus on defining high-quality tools that the Agent can use.

## Getting Started

### 1. Define your Manifest
Every app must have an `mcp-manifest.json` describing its tools.

```json
{
  "app_id": "com.example.weather",
  "tools": [
    {
      "name": "weather.get_forecast",
      "description": "Get the weather forecast for a location.",
      "parameters": {
        "location": { "type": "string" }
      }
    }
  ]
}
```

### 2. Implement the Tool Handler (Kotlin)
Use the `McpToolProvider` base class to handle requests.

```kotlin
class WeatherProvider : McpToolProvider() {
    override fun onToolInvoke(method: String, params: Map<String, Any>): ToolResult {
        if (method == "weather.get_forecast") {
            val location = params["location"]
            return ToolResult.Success(fetchWeather(location))
        }
        return ToolResult.Error("Method not found")
    }
}
```

### 3. Register with mcpd
Apps register their tools via the `McpBridgeService` on startup.

## UI Components (Optional)
While the Agent handles the primary interaction, apps can provide "Mini-UIs" (Widgets/Cards) that the Agent can display in its chat stream for richer data visualization.

## Best Practices
*   **Semantic Naming**: Use clear, descriptive names for tools and parameters.
*   **Granularity**: Tools should be atomic and perform a single logical action.
*   **Safety**: Explicitly mark high-risk tools (e.g., `account.delete`).
