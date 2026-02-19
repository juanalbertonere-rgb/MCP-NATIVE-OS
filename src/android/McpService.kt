package com.mcp.os.system

import android.app.Service
import android.content.Intent
import android.net.LocalSocket
import android.net.LocalSocketAddress
import android.os.IBinder
import android.util.Log

class McpBridgeService : Service() {

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

    override fun onBind(intent: Intent?): IBinder? {
        return null
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
