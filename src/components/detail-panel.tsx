import { useState } from "react";
import {
  Card,
  CardContent,
  CardDescription,
  CardFooter,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Checkbox } from "@/components/ui/checkbox";
import {
  AlertDialog,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
} from "@/components/ui/alert-dialog";
import { ScrollArea } from "@/components/ui/scroll-area";
import { Skeleton } from "@/components/ui/skeleton";
import {
  Tooltip,
  TooltipContent,
  TooltipTrigger,
} from "@/components/ui/tooltip";
import {
  CircleStopIcon,
  FolderOpenIcon,
  InfoIcon,
  LoaderCircleIcon,
  LockIcon,
  PackageOpenIcon,
  RotateCwIcon,
  Trash2Icon,
} from "lucide-react";
import {
  useAppsStore,
  useRelatedStore,
  useTaskStore,
} from "@/stores/uninstaller";
import { uninstallerApi } from "@/lib/api/uninstaller";
import { UninstallConfirm } from "@/components/uninstall-confirm";
import { toast } from "sonner";
import { IDS, STYLES } from "@/lib/styles";
import { cn, formatBytes, formatTimestamp } from "@/lib/utils";
import { useAppSize } from "@/hooks/use-app-size";
import type { AppInfo } from "@/types/models";

export function DetailPanel() {
  const apps = useAppsStore((s) => s.apps);
  const selectedPath = useAppsStore((s) => s.selectedPath);
  const fetchApps = useAppsStore((s) => s.fetchApps);

  const related = useRelatedStore();
  const taskRunning =
    useTaskStore((s) => s.current && !s.current.finished) ?? false;

  const app = apps.find((a) => a.path === selectedPath) ?? null;

  const { size, loading: sizeLoading } = useAppSize(app?.path ?? null);

  const [confirmOpen, setConfirmOpen] = useState(false);
  const [quitOpen, setQuitOpen] = useState(false);

  if (!app) {
    return (
      <div
        id={IDS.detailEmpty}
        className="flex h-full items-center justify-center text-sm text-muted-foreground"
      >
        <div className="flex flex-col items-center gap-2">
          <PackageOpenIcon className="size-8" />
          <p>Select an app to see details and related files.</p>
        </div>
      </div>
    );
  }

  const allSelected =
    related.paths.length > 0 && related.selected.size === related.paths.length;
  const uninstallDisabled = taskRunning || related.loading || app.running;

  async function handleUninstall() {
    if (!app) return;
    setConfirmOpen(false);
    try {
      const report = await uninstallerApi.uninstall(
        app.path,
        app.name,
        app.bundleId,
        Array.from(related.selected),
      );
      toast.success(
        `Removed ${report.removed.length} item${report.removed.length === 1 ? "" : "s"}`,
        {
          description:
            report.failed.length > 0
              ? `${report.failed.length} item(s) could not be removed.`
              : undefined,
        },
      );
      related.clear();
      await fetchApps();
    } catch (e) {
      toast.error("Uninstall failed", { description: String(e) });
    }
  }

  async function handleQuit() {
    if (!app) return;
    setQuitOpen(false);
    try {
      const killed = await uninstallerApi.killApp(
        app.path,
        app.name,
        app.bundleId,
      );
      if (killed > 0) {
        toast.success(`Quit ${app.name}`, {
          description: `Sent SIGKILL to ${killed} process${killed === 1 ? "" : "es"}.`,
        });
      } else {
        toast.warning(`No processes matched ${app.name}`);
      }
      await fetchApps();
    } catch (e) {
      toast.error("Quit failed", { description: String(e) });
    }
  }

  return (
    <div
      id={IDS.detail}
      className="flex h-full flex-col gap-4 overflow-hidden p-4"
    >
      <AppCard
        app={app}
        size={size}
        sizeLoading={sizeLoading}
        uninstallDisabled={uninstallDisabled}
        rescanDisabled={related.loading || taskRunning}
        onReveal={() => uninstallerApi.revealInFinder(app.path)}
        onRescan={() => related.fetchRelated(app.name, app.bundleId)}
        onUninstall={() => setConfirmOpen(true)}
        onQuit={() => setQuitOpen(true)}
      />

      <Card id={IDS.relatedCard} className={STYLES.flexFillCard}>
        <CardHeader>
          <CardTitle>Related files</CardTitle>
          <CardDescription>
            Caches, preferences, application support, containers, logs, and
            launch agents that match this app's name or bundle id.
          </CardDescription>
        </CardHeader>
        <CardContent className="flex flex-1 min-h-0 flex-col gap-3">
          {related.loading ? (
            <div className="flex flex-col gap-1.5">
              {Array.from({ length: 6 }).map((_, i) => (
                <Skeleton key={i} className="h-7 w-full" />
              ))}
            </div>
          ) : related.paths.length === 0 ? (
            <p className="text-sm text-muted-foreground">
              No related files were found.
            </p>
          ) : (
            <>
              <div id={IDS.relatedScanNotice} className={STYLES.infoBanner}>
                <InfoIcon className="size-3.5 shrink-0" />
                <p>
                  These items are matched by the app's name and bundle id, so
                  some may belong to other apps. Review each path carefully
                  before uninstalling.
                </p>
              </div>

              <label
                id={IDS.relatedSelectAll}
                className="flex items-center gap-2 text-sm cursor-pointer select-none"
              >
                <Checkbox
                  checked={allSelected}
                  onCheckedChange={(checked) =>
                    related.toggleAll(checked === true)
                  }
                />
                <span>
                  Select all
                  <span className="ml-1 text-muted-foreground">
                    ({related.selected.size}/{related.paths.length})
                  </span>
                </span>
              </label>
              <ScrollArea id={IDS.relatedList} className={STYLES.flexFillList}>
                <ul className="flex flex-col">
                  {related.paths.map((path) => (
                    <PathRow
                      key={path}
                      path={path}
                      checked={related.selected.has(path)}
                      onToggle={() => related.toggle(path)}
                    />
                  ))}
                </ul>
              </ScrollArea>
            </>
          )}
        </CardContent>
      </Card>

      <UninstallConfirm
        open={confirmOpen}
        onOpenChange={setConfirmOpen}
        app={app}
        selectedCount={related.selected.size}
        onConfirm={handleUninstall}
        busy={taskRunning}
      />

      <AlertDialog open={quitOpen} onOpenChange={setQuitOpen}>
        <AlertDialogContent id={IDS.quitDialog}>
          <AlertDialogHeader>
            <AlertDialogTitle>Quit {app.name}?</AlertDialogTitle>
            <AlertDialogDescription>
              All processes belonging to {app.name} will be sent SIGKILL.
              Unsaved work will be lost.
            </AlertDialogDescription>
          </AlertDialogHeader>
          <AlertDialogFooter>
            <Button
              id={IDS.quitCancel}
              variant="outline"
              onClick={() => setQuitOpen(false)}
            >
              Cancel
            </Button>
            <Button
              id={IDS.quitConfirm}
              variant="destructive"
              onClick={handleQuit}
            >
              Quit
            </Button>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>
    </div>
  );
}

function AppCard({
  app,
  size,
  sizeLoading,
  uninstallDisabled,
  rescanDisabled,
  onReveal,
  onRescan,
  onUninstall,
  onQuit,
}: {
  app: AppInfo;
  size: number | null;
  sizeLoading: boolean;
  uninstallDisabled: boolean;
  rescanDisabled: boolean;
  onReveal: () => void;
  onRescan: () => void;
  onUninstall: () => void;
  onQuit: () => void;
}) {
  return (
    <Card id={IDS.detailAppCard}>
      <CardHeader>
        <CardTitle>Application info</CardTitle>
        <CardDescription>macOS application bundle metadata</CardDescription>
      </CardHeader>
      <CardContent className="flex flex-col gap-3">
        {app.running && (
          <div id={IDS.detailRunningWarning} className={STYLES.warningBanner}>
            <LockIcon className="size-3.5 shrink-0" />
            <p>
              <span className="font-medium">{app.name}</span> is currently
              running and cannot be uninstalled. Quit the app first.
            </p>
          </div>
        )}

        <FieldRow id={IDS.detailBundleId} label="Bundle ID">
          {app.bundleId ?? <Muted>none</Muted>}
        </FieldRow>
        <FieldRow id={IDS.detailVersion} label="Version">
          {app.version ?? <Muted>unknown</Muted>}
        </FieldRow>
        <FieldRow id={IDS.detailExecutable} label="Executable">
          {app.executable ?? <Muted>unknown</Muted>}
        </FieldRow>
        <FieldRow id={IDS.detailSize} label="Size">
          {sizeLoading ? (
            <span className="inline-flex items-center gap-1.5 text-muted-foreground">
              <LoaderCircleIcon className="size-3 animate-spin" />
              Calculating…
            </span>
          ) : (
            formatBytes(size)
          )}
        </FieldRow>
        <FieldRow id={IDS.detailModified} label="Last modified">
          {formatTimestamp(app.modifiedAt)}
        </FieldRow>
        <FieldRow id={IDS.detailPath} label="Path">
          {app.path}
        </FieldRow>
      </CardContent>
      <CardFooter className="justify-end gap-2">
        {app.running && (
          <Button
            id={IDS.detailQuit}
            variant="destructive"
            size="sm"
            className="mr-auto"
            onClick={onQuit}
          >
            <CircleStopIcon />
            Quit
          </Button>
        )}
        <Button
          id={IDS.detailReveal}
          variant="outline"
          size="sm"
          onClick={onReveal}
        >
          <FolderOpenIcon />
          Reveal
        </Button>
        <Button
          id={IDS.detailRescan}
          variant="outline"
          size="sm"
          onClick={onRescan}
          disabled={rescanDisabled}
        >
          <RotateCwIcon />
          Scan files
        </Button>
        <Button
          id={IDS.detailUninstall}
          variant="destructive"
          size="sm"
          onClick={onUninstall}
          disabled={uninstallDisabled}
        >
          {app.running ? <LockIcon /> : <Trash2Icon />}
          Uninstall
        </Button>
      </CardFooter>
    </Card>
  );
}

function FieldRow({
  id,
  label,
  children,
}: {
  id: string;
  label: string;
  children: React.ReactNode;
}) {
  return (
    <div id={id} className="flex items-baseline gap-3">
      <span className={STYLES.fieldRowLabel}>{label}</span>
      <span className={STYLES.fieldRowValue}>{children}</span>
    </div>
  );
}

function Muted({ children }: { children: React.ReactNode }) {
  return <span className="italic text-muted-foreground">{children}</span>;
}

function PathRow({
  path,
  checked,
  onToggle,
}: {
  path: string;
  checked: boolean;
  onToggle: () => void;
}) {
  const checkboxId = IDS.relatedRowCheckbox(path);
  return (
    <li
      id={IDS.relatedRow(path)}
      className={cn("flex items-center gap-2 px-2 py-1", STYLES.rowHover)}
    >
      <Checkbox id={checkboxId} checked={checked} onCheckedChange={onToggle} />
      <Tooltip>
        <TooltipTrigger
          render={
            <label
              htmlFor={checkboxId}
              className="flex-1 truncate font-mono text-xs cursor-pointer select-none"
            >
              {path}
            </label>
          }
        />
        <TooltipContent className="max-w-md">
          <span className="font-mono text-xs break-all">{path}</span>
        </TooltipContent>
      </Tooltip>
      <Tooltip>
        <TooltipTrigger
          render={
            <Button
              id={IDS.relatedRowMenu(path)}
              variant="ghost"
              size="icon-xs"
              aria-label="Reveal in Finder"
              onClick={() => uninstallerApi.revealInFinder(path)}
            >
              <FolderOpenIcon />
            </Button>
          }
        />
        <TooltipContent>Reveal in Finder</TooltipContent>
      </Tooltip>
    </li>
  );
}
