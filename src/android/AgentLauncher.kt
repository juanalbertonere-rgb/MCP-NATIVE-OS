package com.mcp.os.launcher

import android.os.Bundle
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.compose.material3.*
import androidx.compose.runtime.*

class AgentLauncherActivity : ComponentActivity() {
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        setContent {
            AgentLauncherScreen()
        }
    }
}

@Composable
fun AgentLauncherScreen() {
    var userInput by remember { mutableStateOf("") }
    var agentStatus by remember { mutableStateOf("Ready to assist") }

    Column {
        Text("Agent-First OS", style = MaterialTheme.typography.headlineLarge)

        // Agent Thought/Action Stream
        Surface(modifier = Modifier.weight(1f)) {
            Text("Agent Status: $agentStatus")
        }

        // Input Area
        TextField(
            value = userInput,
            onValueChange = { userInput = it },
            placeholder = { Text("Ask the agent anything...") }
        )

        Button(onClick = {
            agentStatus = "Planning: Sending photo to Mom..."
            // TODO: Send input to Agent Runtime
        }) {
            Text("Execute Intent")
        }

        // Minimal Fallback
        TextButton(onClick = { /* Launch legacy app grid */ }) {
            Text("Manual App Launcher (Fallback)")
        }
    }
}
