package com.mcp.os.system

import android.app.Service
import android.content.Intent
import android.net.LocalSocket
import android.net.LocalSocketAddress
import android.os.Binder
import android.os.IBinder
import android.util.Log

class McpBridgeService : Service() {

    inner class McpBinder : Binder() {
        fun getService(): McpBridgeService = this@McpBridgeService
    }

    private val binder = McpBinder()

    override fun onCreate() {
        super.onCreate()
        Log.i("McpService", "Connecting to mcpd via UDS...")
        connectToDaemon()
    }

    private var socket: LocalSocket? = null

    private fun connectToDaemon() {
        try {
            socket = LocalSocket()
            socket?.connect(LocalSocketAddress("/tmp/mcpd.sock", LocalSocketAddress.Namespace.FILESYSTEM))
            Log.i("McpService", "Bridge established: Android Services <-> mcpd via /tmp/mcpd.sock")
            registerAndroidTools()
        } catch (e: Exception) {
            Log.e("McpService", "Failed to connect to mcpd: ${e.message}")
        }
    }

    override fun onBind(intent: Intent?): IBinder {
        return binder
    }

    fun executeIntent(userInput: String) {
        Log.i("McpService", "Executing intent from UI: $userInput")
        // In a real implementation, this would send a message to the AgentOrchestrator
        // For now, we simulate sending a request to mcpd
        sendRequestToDaemon(userInput)
    }

    private fun sendRequestToDaemon(input: String) {
        try {
            val request = """
                {
                    "jsonrpc": "2.0",
                    "method": "agent.process",
                    "params": { "input": "$input" },
                    "id": ${System.currentTimeMillis()},
                    "context": { "mcpd_version": "1.0" }
                }
            """.trimIndent().replace("\n", "") + "\n"
            socket?.outputStream?.write(request.toByteArray())
            socket?.outputStream?.flush()
        } catch (e: Exception) {
            Log.e("McpService", "Failed to send request: ${e.message}")
        }
    }

    /**
     * Expose Android-specific capabilities as MCP tools
     */
    fun registerAndroidTools() {
        val tools = listOf(
            "camera.capture",
            "contacts.resolve",
            "messages.send"
        )
        tools.forEach { tool ->
            Log.i("McpService", "Registering Android capability: $tool")
            sendRegistrationRequest(tool)
        }
    }

    private fun sendRegistrationRequest(toolName: String) {
        try {
            val request = """
                {
                    "jsonrpc": "2.0",
                    "method": "system.register_tool",
                    "params": {
                        "name": "$toolName",
                        "provider": "android.system",
                        "risk_level": "Medium"
                    },
                    "id": ${System.currentTimeMillis()}
                }
            """.trimIndent().replace("\n", "") + "\n"

            socket?.outputStream?.write(request.toByteArray())
            socket?.outputStream?.flush()
        } catch (e: Exception) {
            Log.e("McpService", "Failed to register tool $toolName: ${e.message}")
        }
    }
}
