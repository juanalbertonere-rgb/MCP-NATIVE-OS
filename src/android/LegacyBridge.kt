package com.mcp.os.bridge

import android.accessibilityservice.AccessibilityService
import android.net.LocalSocket
import android.net.LocalSocketAddress
import android.util.Log
import android.view.accessibility.AccessibilityEvent
import android.view.accessibility.AccessibilityNodeInfo

class LegacyAppBridgeService : AccessibilityService() {

    private var socket: LocalSocket? = null

    override fun onServiceConnected() {
        super.onServiceConnected()
        try {
            socket = LocalSocket()
            socket?.connect(LocalSocketAddress("/tmp/mcpd.sock", LocalSocketAddress.Namespace.FILESYSTEM))
        } catch (e: Exception) {
            Log.e("LegacyBridge", "Failed to connect to mcpd: ${e.message}")
        }
    }

    override fun onAccessibilityEvent(event: AccessibilityEvent) {
        // Scrape UI tree and update mcpd tool registry
        val rootNode = rootInActiveWindow ?: return
        val tools = mapUiToTools(rootNode)

        notifyMcpd(tools)
    }

    private fun notifyMcpd(tools: List<VirtualTool>) {
        synchronized(this) {
            tools.forEach { tool ->
                try {
                    val request = """
                    {
                        "jsonrpc": "2.0",
                        "method": "system.register_tool",
                        "params": {
                            "name": "${tool.name}",
                            "provider": "legacy.bridge",
                            "risk_level": "Low"
                        },
                        "id": ${System.currentTimeMillis()}
                    }
                """.trimIndent().replace("\n", "") + "\n"

                socket?.outputStream?.write(request.toByteArray())
                socket?.outputStream?.flush()
                    } catch (e: Exception) {
                        Log.e("LegacyBridge", "Failed to notify mcpd of tool ${tool.name}: ${e.message}")
                    }
            }
        }
    }

    private fun mapUiToTools(node: AccessibilityNodeInfo): List<VirtualTool> {
        val tools = mutableListOf<VirtualTool>()
        // Recursively find clickable elements and extract labels
        if (node.isClickable && node.text != null) {
            tools.add(VirtualTool(
                name = "legacy.click_${node.text.toString().lowercase().replace(" ", "_")}",
                nodeInfo = node
            ))
        }
        for (i in 0 until node.childCount) {
            tools.addAll(mapUiToTools(node.getChild(i)))
        }
        return tools
    }

    override fun onInterrupt() {}
}

data class VirtualTool(val name: String, val nodeInfo: AccessibilityNodeInfo)
