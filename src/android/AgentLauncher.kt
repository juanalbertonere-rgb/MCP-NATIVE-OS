package com.mcp.os.launcher

import android.content.ComponentName
import android.content.Context
import android.content.Intent
import android.content.ServiceConnection
import android.os.Bundle
import android.os.IBinder
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.compose.material3.*
import androidx.compose.runtime.*
import com.mcp.os.system.McpBridgeService

class AgentLauncherActivity : ComponentActivity() {
    private var mcpService: McpBridgeService? = null
    private var isBound = false

    private val connection = object : ServiceConnection {
        override fun onServiceConnected(name: ComponentName?, service: IBinder?) {
            val binder = service as McpBridgeService.McpBinder
            mcpService = binder.getService()
            isBound = true
        }

        override fun onServiceDisconnected(name: ComponentName?) {
            isBound = false
        }
    }

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        Intent(this, McpBridgeService::class.java).also { intent ->
            bindService(intent, connection, Context.BIND_AUTO_CREATE)
        }
        setContent {
            AgentLauncherScreen(mcpService)
        }
    }

    override fun onDestroy() {
        super.onDestroy()
        if (isBound) {
            unbindService(connection)
            isBound = false
        }
    }
}

@Composable
fun AgentLauncherScreen(mcpService: McpBridgeService?) {
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
            agentStatus = "Planning: $userInput..."
            mcpService?.executeIntent(userInput)
        }) {
            Text("Execute Intent")
        }

        // Minimal Fallback
        TextButton(onClick = { /* Launch legacy app grid */ }) {
            Text("Manual App Launcher (Fallback)")
        }
    }
}
