import React, { useState, useEffect, useRef } from "react";
import { Button, Header, Input, Switch } from "@/components";
import {
  KeyIcon,
  TrashIcon,
  LoaderIcon,
  ChevronDown,
  CoffeeIcon,
} from "lucide-react";
import { invoke } from "@tauri-apps/api/core";
import { openUrl } from "@tauri-apps/plugin-opener";
import { useApp } from "@/contexts";
import {
  Command,
  CommandEmpty,
  CommandGroup,
  CommandInput,
  CommandItem,
  CommandList,
  Popover,
  PopoverContent,
  PopoverTrigger,
} from "@/components";

interface ActivationResponse {
  activated: boolean;
  error?: string;
  license_key?: string;
  instance?: {
    id: string;
    name: string;
    created_at: string;
  };
}

interface CheckoutResponse {
  success?: boolean;
  checkout_url?: string;
  error?: string;
}

interface StorageResult {
  license_key?: string;
  instance_id?: string;
  selected_extab_model?: string;
}

interface Model {
  provider: string;
  name: string;
  id: string;
  model: string;
  description: string;
  modality: string;
  isAvailable: boolean;
}

const LICENSE_KEY_STORAGE_KEY = "extab_license_key";
const INSTANCE_ID_STORAGE_KEY = "extab_instance_id";
const SELECTED_EXTAB_MODEL_STORAGE_KEY = "selected_extab_model";

export const ExtabApiSetup = () => {
  const { extabApiEnabled, setExtabApiEnabled } = useApp();

  const [licenseKey, setLicenseKey] = useState("");
  const [storedLicenseKey, setStoredLicenseKey] = useState<string | null>(null);
  const [maskedLicenseKey, setMaskedLicenseKey] = useState<string | null>(null);
  const [isLoading, setIsLoading] = useState(false);
  const [isCheckoutLoading, setIsCheckoutLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [success, setSuccess] = useState<string | null>(null);
  const [models, setModels] = useState<Model[]>([]);
  const [isModelsLoading, setIsModelsLoading] = useState(false);
  const [selectedModel, setSelectedModel] = useState<Model | null>(null);
  const [isPopoverOpen, setIsPopoverOpen] = useState(false);
  const [searchValue, setSearchValue] = useState("");
  const fetchInitiated = useRef(false);
  const commandListRef = useRef<HTMLDivElement>(null);

  // Load license status on component mount
  useEffect(() => {
    loadLicenseStatus();
    if (!fetchInitiated.current) {
      fetchInitiated.current = true;
      fetchModels();
    }
  }, []);

  // Scroll to top when search value changes
  useEffect(() => {
    if (commandListRef.current) {
      commandListRef.current.scrollTop = 0;
    }
  }, [searchValue]);

  const fetchModels = async () => {
    setIsModelsLoading(true);
    try {
      const fetchedModels = await invoke<Model[]>("fetch_models");
      setModels(fetchedModels);
    } catch (error) {
      console.error("Failed to fetch models:", error);
    } finally {
      setIsModelsLoading(false);
    }
  };

  const loadLicenseStatus = async () => {
    try {
      // Get all stored data in one call
      const storage = await invoke<StorageResult>("secure_storage_get");

      if (storage.license_key) {
        setStoredLicenseKey(storage.license_key);

        // Get masked version from Tauri command
        const masked = await invoke<string>("mask_license_key_cmd", {
          licenseKey: storage.license_key,
        });
        setMaskedLicenseKey(masked);
      } else {
        setStoredLicenseKey(null);
        setMaskedLicenseKey(null);
      }

      if (storage.selected_extab_model) {
        try {
          const storedModel = JSON.parse(storage.selected_extab_model);
          setSelectedModel(storedModel);
        } catch (e) {
          console.error("Failed to parse stored model:", e);
          setSelectedModel(null);
        }
      } else {
        setSelectedModel(null);
      }
    } catch (err) {
      console.error("Failed to load license status:", err);
      // If we can't read from storage, assume no license is stored
      setStoredLicenseKey(null);
      setMaskedLicenseKey(null);
      setSelectedModel(null);
    }
  };

  const handleActivateLicense = async () => {
    if (!licenseKey.trim()) {
      setError("Please enter a license key");
      return;
    }

    setIsLoading(true);
    setError(null);
    setSuccess(null);

    try {
      const response: ActivationResponse = await invoke(
        "activate_license_api",
        {
          licenseKey: licenseKey.trim(),
        }
      );

      if (response.activated && response.instance) {
        // Store the license data securely in one call
        await invoke("secure_storage_save", {
          items: [
            {
              key: LICENSE_KEY_STORAGE_KEY,
              value: licenseKey.trim(),
            },
            {
              key: INSTANCE_ID_STORAGE_KEY,
              value: response.instance.id,
            },
          ],
        });

        setSuccess("License activated successfully!");
        setLicenseKey(""); // Clear the input

        // Auto-enable Extab API when license is activated
        setExtabApiEnabled(true);

        await loadLicenseStatus(); // Reload status
      } else {
        setError(response.error || "Failed to activate license");
      }
    } catch (err) {
      console.error("License activation failed:", err);
      setError(typeof err === "string" ? err : "Failed to activate license");
    } finally {
      setIsLoading(false);
    }
  };

  const handleRemoveLicense = async () => {
    setIsLoading(true);
    setError(null);
    setSuccess(null);

    try {
      // Remove all license data from secure storage in one call
      await invoke("secure_storage_remove", {
        keys: [
          LICENSE_KEY_STORAGE_KEY,
          INSTANCE_ID_STORAGE_KEY,
          SELECTED_EXTAB_MODEL_STORAGE_KEY,
        ],
      });

      setSuccess("License removed successfully!");

      // Disable Extab API when license is removed
      setExtabApiEnabled(false);

      await loadLicenseStatus(); // Reload status
    } catch (err) {
      console.error("Failed to remove license:", err);
      setError("Failed to remove license");
    } finally {
      setIsLoading(false);
    }
  };

  const handleModelSelect = async (model: Model) => {
    setSelectedModel(model);
    setIsPopoverOpen(false); // Close popover when model is selected
    setSearchValue(""); // Reset search when model is selected
    try {
      await invoke("secure_storage_save", {
        items: [
          {
            key: SELECTED_EXTAB_MODEL_STORAGE_KEY,
            value: JSON.stringify(model),
          },
        ],
      });
    } catch (error) {
      console.error("Failed to save model selection:", error);
      setError("Failed to save model selection.");
    }
  };

  const handlePopoverOpenChange = (open: boolean) => {
    setIsPopoverOpen(open);
    if (open) {
      setSearchValue(""); // Reset search when popover opens
    }
  };

  const handleKeyDown = (e: React.KeyboardEvent<HTMLInputElement>) => {
    if (e.key === "Enter" && !storedLicenseKey) {
      handleActivateLicense();
    }
  };

  const handleGetLicenseKey = async () => {
    setIsCheckoutLoading(true);
    setError(null);

    try {
      const response: CheckoutResponse = await invoke("get_checkout_url");

      if (response.success && response.checkout_url) {
        // Open checkout URL in default browser
        await openUrl(response.checkout_url);
      } else {
        setError(response.error || "Failed to get checkout URL");
      }
    } catch (err) {
      console.error("Failed to get checkout URL:", err);
      setError(typeof err === "string" ? err : "Failed to get checkout URL");
    } finally {
      setIsCheckoutLoading(false);
    }
  };

  const providers = [...new Set(models.map((model) => model.provider))];
  const capitalizedProviders = providers.map(
    (p) => p.charAt(0).toUpperCase() + p.slice(1)
  );

  let providerList;
  if (capitalizedProviders.length === 0) {
    providerList = null;
  } else if (capitalizedProviders.length === 1) {
    providerList = capitalizedProviders[0];
  } else if (capitalizedProviders.length === 2) {
    providerList = capitalizedProviders.join(" and ");
  } else {
    const lastProvider = capitalizedProviders.pop();
    providerList = `${capitalizedProviders.join(", ")}, and ${lastProvider}`;
  }

  const title = isModelsLoading
    ? "Loading Models..."
    : `Extab supports ${models?.length} model${
        models?.length !== 1 ? "s" : ""
      }`;

  const description = isModelsLoading
    ? "Fetching the list of supported models..."
    : providerList
    ? `Access top models from providers like ${providerList}. and select smaller models for faster responses.`
    : "Explore all the models Extab supports.";

  return (
    <div className="space-y-3">
      <div className="space-y-2 flex flex-row items-center justify-between border-b pb-2">
        <Header
          title="Support Extab"
          description="Support Extab to keep the project alive, and follow on X."
        />
        <div className="flex flex-row items-center gap-2">
          <Button
            size="sm"
            variant="outline"
            onClick={() => openUrl("https://x.com/shouryamaanjain")}
          >
            <svg width="16" height="16" fill="currentColor" viewBox="0 0 16 16">
              <path d="M12.6.75h2.454l-5.36 6.142L16 15.25h-4.937l-3.867-5.07-4.425 5.07H.316l5.733-6.57L0 .75h5.063l3.495 4.633L12.601.75Zm-.86 13.028h1.36L4.323 2.145H2.865z" />
            </svg>
          </Button>
          <Button
            size="sm"
            variant="outline"
            onClick={() => openUrl("https://extab.com/feedback")}
          >
            Feedback
          </Button>
          <Button
            size="sm"
            variant="outline"
            onClick={() => openUrl("https://buymeacoffee.com/shouryamaanjain")}
          >
            <CoffeeIcon className="h-4 w-4" /> Buy Me a Coffee
          </Button>
        </div>
      </div>

      <div className="space-y-2 pt-2">
        <div className="flex items-center justify-between border-b pb-2">
          <Header
            titleClassName="text-lg"
            title="Extab Access"
            description="Extab license to unlock faster responses, quicker support and premium features."
          />
          <div className="flex flex-row items-center gap-2">
            {!storedLicenseKey && (
              <Button
                onClick={handleGetLicenseKey}
                disabled={isCheckoutLoading}
                size="sm"
              >
                {isCheckoutLoading ? "Loading..." : "Get License Key"}
              </Button>
            )}
          </div>
        </div>

        {/* Error Message */}
        {error && (
          <div className="p-3 rounded-lg border border-red-200 bg-red-50 dark:border-red-800 dark:bg-red-950">
            <p className="text-sm text-red-700 dark:text-red-400">{error}</p>
          </div>
        )}

        {/* Success Message */}
        {success && (
          <div className="p-3 rounded-lg border border-green-200 bg-green-50 dark:border-green-800 dark:bg-green-950">
            <p className="text-sm text-green-700 dark:text-green-400">
              {success}
            </p>
          </div>
        )}
        <Header title={title} description={description} />
        <Popover
          modal={true}
          open={isPopoverOpen}
          onOpenChange={handlePopoverOpenChange}
        >
          <PopoverTrigger
            asChild
            disabled={isModelsLoading}
            className="cursor-pointer flex justify-start"
          >
            <Button
              variant="outline"
              className="h-11 text-start shadow-none w-full"
            >
              {selectedModel ? selectedModel.name : "Select pro models"}{" "}
              <ChevronDown />
            </Button>
          </PopoverTrigger>
          <PopoverContent
            align="end"
            side="bottom"
            className="w-[calc(100vw-4rem)] h-[46vh]"
          >
            <Command shouldFilter={true}>
              <CommandInput
                placeholder="Select model..."
                value={searchValue}
                onValueChange={setSearchValue}
              />
              <CommandList
                ref={commandListRef}
                className="overflow-y-auto [&::-webkit-scrollbar]:w-2 [&::-webkit-scrollbar-track]:rounded-full [&::-webkit-scrollbar-track]:bg-muted [&::-webkit-scrollbar-thumb]:rounded-full [&::-webkit-scrollbar-thumb]:bg-muted-foreground/20 [&::-webkit-scrollbar-thumb:hover]:bg-muted-foreground/30"
              >
                <CommandEmpty>
                  No models found. Please try again later.
                </CommandEmpty>
                <CommandGroup>
                  {models.map((model, index) => (
                    <CommandItem
                      disabled={!model?.isAvailable}
                      key={`${model?.id}-${index}`}
                      className="cursor-pointer"
                      onSelect={() => handleModelSelect(model)}
                    >
                      <div className="flex flex-col">
                        <div className="flex flex-row items-center gap-2">
                          <p className="text-sm font-medium">{`${model?.name} - (${model?.modality})`}</p>
                          {model?.isAvailable ? (
                            <div className="text-xs text-orange-600 bg-white rounded-full px-2">
                              {model?.provider}
                            </div>
                          ) : (
                            <div className="text-xs text-red-600 bg-white rounded-full px-2">
                              Not Available
                            </div>
                          )}
                        </div>
                        <p
                          className="text-sm text-muted-foreground line-clamp-2"
                          title={model?.description}
                        >
                          {model?.description}
                        </p>
                      </div>
                    </CommandItem>
                  ))}
                </CommandGroup>
              </CommandList>
            </Command>
          </PopoverContent>
        </Popover>
        {/* License Key Input or Display */}
        <div className="space-y-2">
          {!storedLicenseKey ? (
            <>
              <div className="space-y-1">
                <label className="text-sm font-medium">License Key</label>
                <p className="text-sm font-medium text-muted-foreground">
                  After completing your purchase, you'll receive a license key
                  via email. Paste it below to activate.
                </p>
              </div>
              <div className="flex gap-2">
                <Input
                  type="password"
                  placeholder="Enter your license key (e.g., 38b1460a-5104-4067-a91d-77b872934d51)"
                  value={licenseKey}
                  onChange={(value) => {
                    setLicenseKey(
                      typeof value === "string" ? value : value.target.value
                    );
                    setError(null); // Clear error when user types
                    setSuccess(null); // Clear success when user types
                  }}
                  onKeyDown={handleKeyDown}
                  disabled={isLoading}
                  className="flex-1 h-11 border-1 border-input/50 focus:border-primary/50 transition-colors"
                />
                <Button
                  onClick={handleActivateLicense}
                  disabled={isLoading || !licenseKey.trim()}
                  size="icon"
                  className="shrink-0 h-11 w-11"
                  title="Activate License"
                >
                  {isLoading ? (
                    <LoaderIcon className="h-4 w-4 animate-spin" />
                  ) : (
                    <KeyIcon className="h-4 w-4" />
                  )}
                </Button>
              </div>
            </>
          ) : (
            <>
              <label className="text-sm font-medium">Current License</label>
              <div className="flex gap-2">
                <Input
                  type="text"
                  value={maskedLicenseKey || ""}
                  disabled={true}
                  className="flex-1 h-11 border-1 border-input/50 bg-muted/50"
                />
                <Button
                  onClick={handleRemoveLicense}
                  disabled={isLoading}
                  size="icon"
                  variant="destructive"
                  className="shrink-0 h-11 w-11"
                  title="Remove License"
                >
                  {isLoading ? (
                    <LoaderIcon className="h-4 w-4 animate-spin" />
                  ) : (
                    <TrashIcon className="h-4 w-4" />
                  )}
                </Button>
              </div>
              {storedLicenseKey ? (
                <div className="-mt-1">
                  <p className="text-sm font-medium text-muted-foreground select-auto">
                    If you need any help or any assistance, contact
                    support@extab.com
                  </p>
                </div>
              ) : null}
            </>
          )}
        </div>
      </div>
      <div className="flex justify-between items-center">
        <Header
          title={`${extabApiEnabled ? "Disable" : "Enable"} Extab API`}
          description={
            storedLicenseKey
              ? extabApiEnabled
                ? "Using all extab APIs for audio, and chat."
                : "Using all your own AI Providers for audio, and chat."
              : "A valid license is required to enable Extab API or you can use your own AI Providers and STT Providers."
          }
        />
        <Switch
          checked={extabApiEnabled}
          onCheckedChange={setExtabApiEnabled}
          disabled={!storedLicenseKey} // Disable if no license is stored
        />
      </div>
    </div>
  );
};
