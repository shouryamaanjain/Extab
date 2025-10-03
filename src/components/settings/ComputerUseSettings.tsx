import {
  Label,
  Input,
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  Switch,
  Header,
} from "@/components";
import { useApp } from "@/contexts";

export const ComputerUseSettings = () => {
  const { computerUseConfig, setComputerUseConfig } = useApp();

  const handleEnabledChange = (enabled: boolean) => {
    setComputerUseConfig((prev) => ({ ...prev, enabled }));
  };

  const handleDisplayWidthChange = (value: string) => {
    const width = parseInt(value);
    if (!isNaN(width) && width > 0) {
      setComputerUseConfig((prev) => ({ ...prev, displayWidth: width }));
    }
  };

  const handleDisplayHeightChange = (value: string) => {
    const height = parseInt(value);
    if (!isNaN(height) && height > 0) {
      setComputerUseConfig((prev) => ({ ...prev, displayHeight: height }));
    }
  };

  const handleModelChange = (model: string) => {
    setComputerUseConfig((prev) => ({ ...prev, model }));
  };

  const handleMaxIterationsChange = (value: string) => {
    const iterations = parseInt(value);
    if (!isNaN(iterations) && iterations > 0) {
      setComputerUseConfig((prev) => ({ ...prev, maxIterations: iterations }));
    }
  };

  const handleLiveFeedbackChange = (showLiveFeedback: boolean) => {
    setComputerUseConfig((prev) => ({ ...prev, showLiveFeedback }));
  };

  return (
    <div className="space-y-4">
      <Header
        title="Computer Use (Beta)"
        description="Allow Claude to control your computer - interact with applications, click, type, and automate tasks."
        isMainTitle
      />

      {/* Warning Banner */}
      <div className="p-3 bg-yellow-500/10 border border-yellow-500/30 rounded-md">
        <div className="flex items-start gap-2">
          <span className="text-yellow-500 text-lg">⚠️</span>
          <div className="flex-1 text-xs text-yellow-600 dark:text-yellow-400">
            <p className="font-semibold mb-1">Important Security Notice</p>
            <p>
              Computer use gives Claude full control of your mouse and keyboard. Only enable this feature when you trust the tasks you're asking Claude to perform. Always monitor Claude's actions.
            </p>
          </div>
        </div>
      </div>

      {/* Enable/Disable Computer Use */}
      <div className="flex items-center justify-between">
        <Header
          title="Enable Computer Use"
          description="Allow Claude to control your computer"
        />
        <Switch
          checked={computerUseConfig.enabled}
          onCheckedChange={handleEnabledChange}
        />
      </div>

      {/* Configuration Options - Only show when enabled */}
      {computerUseConfig.enabled && (
        <div className="space-y-4 pt-2">
          {/* Model Selection */}
          <div className="space-y-2">
            <Header
              title="Claude Model"
              description="Model to use for computer use (must support computer use tools)"
            />
            <Select value={computerUseConfig.model} onValueChange={handleModelChange}>
              <SelectTrigger className="w-full h-11 border-1 border-input/50 focus:border-primary/50 transition-colors">
                <div className="flex items-center gap-2">
                  <div className="text-sm font-medium">{computerUseConfig.model}</div>
                </div>
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="claude-sonnet-4-5-20250929">
                  <div className="font-medium">Claude Sonnet 4.5</div>
                </SelectItem>
                <SelectItem value="claude-3-7-sonnet-20250219">
                  <div className="font-medium">Claude Sonnet 3.7</div>
                </SelectItem>
                <SelectItem value="claude-3-5-sonnet-20241022">
                  <div className="font-medium">Claude Sonnet 3.5 (deprecated)</div>
                </SelectItem>
              </SelectContent>
            </Select>
            <p className="text-xs text-muted-foreground">
              Recommended: Claude Sonnet 4.5 for best performance
            </p>
          </div>

          {/* Display Dimensions */}
          <div className="space-y-3">
            <Header
              title="Display Dimensions"
              description="Set the virtual display size for Claude (max 1280x800 recommended)"
            />
            <div className="grid grid-cols-2 gap-3">
              <div className="space-y-2">
                <Label className="text-sm font-medium">Width (px)</Label>
                <Input
                  type="number"
                  placeholder="1024"
                  value={computerUseConfig.displayWidth}
                  onChange={(e) => handleDisplayWidthChange(e.target.value)}
                  className="w-full h-11 border-1 border-input/50 focus:border-primary/50 transition-colors"
                  min="640"
                  max="1920"
                />
              </div>
              <div className="space-y-2">
                <Label className="text-sm font-medium">Height (px)</Label>
                <Input
                  type="number"
                  placeholder="768"
                  value={computerUseConfig.displayHeight}
                  onChange={(e) => handleDisplayHeightChange(e.target.value)}
                  className="w-full h-11 border-1 border-input/50 focus:border-primary/50 transition-colors"
                  min="480"
                  max="1080"
                />
              </div>
            </div>
            <p className="text-xs text-muted-foreground">
              Higher resolutions may cause accuracy issues. Keep at or below 1280x800 for best results.
            </p>
          </div>

          {/* Max Iterations */}
          <div className="space-y-2">
            <Header
              title="Max Iterations"
              description="Maximum number of tool use loops before stopping (prevents infinite loops)"
            />
            <Input
              type="number"
              placeholder="10"
              value={computerUseConfig.maxIterations}
              onChange={(e) => handleMaxIterationsChange(e.target.value)}
              className="w-full h-11 border-1 border-input/50 focus:border-primary/50 transition-colors"
              min="1"
              max="50"
            />
            <p className="text-xs text-muted-foreground">
              Each iteration allows Claude to use multiple tools. Default: 10
            </p>
          </div>

          {/* Live Feedback Toggle */}
          <div className="flex items-center justify-between">
            <Header
              title="Show Live Feedback"
              description="Display Claude's actions and thinking process in real-time"
            />
            <Switch
              checked={computerUseConfig.showLiveFeedback}
              onCheckedChange={handleLiveFeedbackChange}
            />
          </div>

          {/* Requirements Notice */}
          <div className="p-3 bg-blue-500/10 border border-blue-500/30 rounded-md">
            <div className="flex items-start gap-2">
              <span className="text-blue-500 text-lg">ℹ️</span>
              <div className="flex-1 text-xs text-blue-600 dark:text-blue-400">
                <p className="font-semibold mb-1">Requirements</p>
                <ul className="list-disc list-inside space-y-1">
                  <li>Anthropic API key configured in AI provider settings</li>
                  <li>Claude Sonnet 4.5, 3.7, or 3.5 v2 model access</li>
                  <li>System permissions for mouse & keyboard control</li>
                </ul>
              </div>
            </div>
          </div>
        </div>
      )}

      {/* Tips */}
      {computerUseConfig.enabled && (
        <div className="text-xs text-muted-foreground/70">
          <p>
            💡 <strong>Tip:</strong> Start with simple tasks like "open Calculator and add 2+2" before trying complex workflows. Always monitor Claude's actions for security.
          </p>
        </div>
      )}
    </div>
  );
};
