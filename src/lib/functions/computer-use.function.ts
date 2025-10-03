// Computer Use Agent Loop
// Implements Claude's computer use feature with tool execution

import { invoke } from "@tauri-apps/api/core";
import { fetch as tauriFetch } from "@tauri-apps/plugin-http";
import { ComputerUseConfig } from "@/types";

// Tool use result type
interface ToolResult {
  type: "tool_result";
  tool_use_id: string;
  content: string | Array<{ type: "image"; source: { type: "base64"; media_type: string; data: string } }>;
  is_error?: boolean;
}

// Message type for Claude API
interface Message {
  role: "user" | "assistant";
  content: string | Array<any>;
}

// Computer use config for API
interface ComputerUseTool {
  type: string;
  name: string;
  display_width_px: number;
  display_height_px: number;
  display_number?: number;
}

// Execute a single computer action
async function executeComputerAction(
  action: string,
  params: any,
  onActionExecuted?: (action: string, params: any) => void
): Promise<ToolResult> {
  try {
    // Notify about action execution
    if (onActionExecuted) {
      onActionExecuted(action, params);
    }

    switch (action) {
      case "screenshot": {
        const base64Image = await invoke<string>("capture_to_base64");
        return {
          type: "tool_result",
          tool_use_id: params.tool_use_id,
          content: [
            {
              type: "image",
              source: {
                type: "base64",
                media_type: "image/jpeg",
                data: base64Image,
              },
            },
          ],
        };
      }

      case "mouse_move": {
        const result = await invoke<string>("computer_mouse_move", {
          x: params.coordinate[0],
          y: params.coordinate[1],
        });
        return {
          type: "tool_result",
          tool_use_id: params.tool_use_id,
          content: result,
        };
      }

      case "left_click": {
        const result = await invoke<string>("computer_mouse_click", {
          x: params.coordinate[0],
          y: params.coordinate[1],
          button: "left",
        });
        return {
          type: "tool_result",
          tool_use_id: params.tool_use_id,
          content: result,
        };
      }

      case "right_click": {
        const result = await invoke<string>("computer_mouse_click", {
          x: params.coordinate[0],
          y: params.coordinate[1],
          button: "right",
        });
        return {
          type: "tool_result",
          tool_use_id: params.tool_use_id,
          content: result,
        };
      }

      case "middle_click": {
        const result = await invoke<string>("computer_mouse_click", {
          x: params.coordinate[0],
          y: params.coordinate[1],
          button: "middle",
        });
        return {
          type: "tool_result",
          tool_use_id: params.tool_use_id,
          content: result,
        };
      }

      case "double_click": {
        const result = await invoke<string>("computer_mouse_double_click", {
          x: params.coordinate[0],
          y: params.coordinate[1],
        });
        return {
          type: "tool_result",
          tool_use_id: params.tool_use_id,
          content: result,
        };
      }

      case "left_click_drag": {
        const result = await invoke<string>("computer_mouse_drag", {
          fromX: params.start_coordinate[0],
          fromY: params.start_coordinate[1],
          toX: params.end_coordinate[0],
          toY: params.end_coordinate[1],
        });
        return {
          type: "tool_result",
          tool_use_id: params.tool_use_id,
          content: result,
        };
      }

      case "scroll": {
        const scrollY = params.scroll_direction === "down" ? params.scroll_amount : -params.scroll_amount;
        const result = await invoke<string>("computer_mouse_scroll", {
          x: params.coordinate?.[0] || 0,
          y: params.coordinate?.[1] || 0,
          scrollX: 0,
          scrollY: scrollY,
        });
        return {
          type: "tool_result",
          tool_use_id: params.tool_use_id,
          content: result,
        };
      }

      case "type": {
        const result = await invoke<string>("computer_keyboard_type", {
          text: params.text,
        });
        return {
          type: "tool_result",
          tool_use_id: params.tool_use_id,
          content: result,
        };
      }

      case "key": {
        const result = await invoke<string>("computer_keyboard_key", {
          key: params.text,
        });
        return {
          type: "tool_result",
          tool_use_id: params.tool_use_id,
          content: result,
        };
      }

      default:
        return {
          type: "tool_result",
          tool_use_id: params.tool_use_id,
          content: `Unsupported action: ${action}`,
          is_error: true,
        };
    }
  } catch (error) {
    console.error(`Error executing ${action}:`, error);
    return {
      type: "tool_result",
      tool_use_id: params.tool_use_id,
      content: `Error: ${error instanceof Error ? error.message : String(error)}`,
      is_error: true,
    };
  }
}

// Computer Use Agent Loop
export async function* computerUseAgentLoop(
  userMessage: string,
  config: ComputerUseConfig,
  apiKey: string,
  systemPrompt?: string,
  onActionExecuted?: (action: string, params: any) => void
): AsyncGenerator<{ type: "text" | "action" | "thinking" | "error" | "complete"; content: string; data?: any }, void, unknown> {
  const messages: Message[] = [
    {
      role: "user",
      content: userMessage,
    },
  ];

  // Define tools for Claude
  const tools: ComputerUseTool[] = [
    {
      type: "computer_20250124",
      name: "computer",
      display_width_px: config.displayWidth,
      display_height_px: config.displayHeight,
      display_number: 1,
    },
  ];

  let iterations = 0;

  try {
    while (iterations < config.maxIterations) {
      iterations++;

      yield {
        type: "action",
        content: `Iteration ${iterations}/${config.maxIterations}`,
        data: { iteration: iterations },
      };

      // Call Claude API using Tauri's HTTP plugin
      const response = await tauriFetch("https://api.anthropic.com/v1/messages", {
        method: "POST",
        headers: {
          "Content-Type": "application/json",
          "x-api-key": apiKey,
          "anthropic-version": "2023-06-01",
          "anthropic-beta": "computer-use-2025-01-24",
          "anthropic-dangerous-direct-browser-access": "true",
        },
        body: JSON.stringify({
          model: config.model,
          max_tokens: 4096,
          messages,
          tools,
          system: systemPrompt,
        }),
      });

      if (!response.ok) {
        const errorText = await response.text();
        yield {
          type: "error",
          content: `API Error: ${response.status} - ${errorText}`,
        };
        return;
      }

      const data = await response.json();

      // Add assistant's response to messages
      messages.push({
        role: "assistant",
        content: data.content,
      });

      // Check for text responses
      for (const block of data.content) {
        if (block.type === "text") {
          yield {
            type: "text",
            content: block.text,
          };
        } else if (block.type === "thinking") {
          yield {
            type: "thinking",
            content: block.thinking,
          };
        }
      }

      // Check if Claude used any tools
      const toolUses = data.content.filter((block: any) => block.type === "tool_use");

      if (toolUses.length === 0) {
        // No more tools to use, task complete
        yield {
          type: "complete",
          content: "Task completed successfully",
        };
        return;
      }

      // Execute all tool uses
      const toolResults: ToolResult[] = [];

      for (const toolUse of toolUses) {
        yield {
          type: "action",
          content: `Executing: ${toolUse.input.action}`,
          data: {
            action: toolUse.input.action,
            params: toolUse.input,
          },
        };

        const result = await executeComputerAction(
          toolUse.input.action,
          { ...toolUse.input, tool_use_id: toolUse.id },
          onActionExecuted
        );

        toolResults.push(result);
      }

      // Add tool results back to messages for next iteration
      messages.push({
        role: "user",
        content: toolResults,
      });

      // Check stop reason
      if (data.stop_reason === "end_turn") {
        yield {
          type: "complete",
          content: "Task completed successfully",
        };
        return;
      }
    }

    // Reached max iterations
    yield {
      type: "error",
      content: `Reached maximum iterations (${config.maxIterations})`,
    };
  } catch (error) {
    console.error("Computer use agent loop error:", error);
    yield {
      type: "error",
      content: `Error: ${error instanceof Error ? error.message : String(error)}`,
    };
  }
}

// Helper to get screen size
export async function getScreenSize(): Promise<{ width: number; height: number }> {
  try {
    const result = await invoke<{ width: number; height: number }>("computer_get_screen_size");
    return result;
  } catch (error) {
    console.error("Failed to get screen size:", error);
    return { width: 1920, height: 1080 }; // Default fallback
  }
}
