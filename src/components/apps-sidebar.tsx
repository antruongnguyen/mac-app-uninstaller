import { useMemo, useState } from "react";
import { LockIcon, SearchIcon, XIcon } from "lucide-react";
import {
  Item,
  ItemActions,
  ItemContent,
  ItemDescription,
  ItemGroup,
  ItemTitle,
} from "@/components/ui/item";
import {
  InputGroup,
  InputGroupAddon,
  InputGroupButton,
  InputGroupInput,
} from "@/components/ui/input-group";
import { ScrollArea } from "@/components/ui/scroll-area";
import { Skeleton } from "@/components/ui/skeleton";
import {
  Tooltip,
  TooltipContent,
  TooltipTrigger,
} from "@/components/ui/tooltip";
import { cn } from "@/lib/utils";
import { IDS } from "@/lib/styles";
import { useIsTruncated } from "@/hooks/use-is-truncated";
import { useAppsStore } from "@/stores/uninstaller";
import type { AppInfo } from "@/types/models";

export function AppsSidebar({
  onSelect,
}: {
  onSelect: (app: AppInfo) => void;
}) {
  const { apps, selectedPath, loading } = useAppsStore();
  const [query, setQuery] = useState("");

  const filtered = useMemo(() => {
    const q = query.trim().toLowerCase();
    if (!q) return apps;
    return apps.filter(
      (a) =>
        a.name.toLowerCase().includes(q) ||
        (a.bundleId ?? "").toLowerCase().includes(q),
    );
  }, [apps, query]);

  return (
    <aside
      id={IDS.sidebar}
      className="flex h-full w-full flex-col border-r bg-sidebar text-sidebar-foreground"
    >
      <div className="shrink-0 p-3">
        <InputGroup>
          <InputGroupAddon>
            <SearchIcon />
          </InputGroupAddon>
          <InputGroupInput
            id={IDS.sidebarSearch}
            placeholder="Search apps..."
            value={query}
            onChange={(e) => setQuery(e.target.value)}
          />
          {query && (
            <InputGroupAddon align="inline-end">
              <InputGroupButton
                size="icon-xs"
                aria-label="Clear search"
                onClick={() => setQuery("")}
              >
                <XIcon />
              </InputGroupButton>
            </InputGroupAddon>
          )}
        </InputGroup>
      </div>

      <ScrollArea id={IDS.sidebarList} className="flex-1 min-h-0 px-2">
        {loading && apps.length === 0 ? (
          <div className="flex flex-col gap-1.5 p-1">
            {Array.from({ length: 10 }).map((_, i) => (
              <Skeleton key={i} className="h-9 w-full" />
            ))}
          </div>
        ) : filtered.length === 0 ? (
          <p className="px-2 py-4 text-center text-xs text-muted-foreground">
            No apps found.
          </p>
        ) : (
          <ItemGroup className="gap-1 pb-2">
            {filtered.map((app) => (
              <AppRow
                key={app.path}
                app={app}
                selected={selectedPath === app.path}
                onSelect={() => onSelect(app)}
              />
            ))}
          </ItemGroup>
        )}
      </ScrollArea>

      <SidebarFooter total={apps.length} />
    </aside>
  );
}

function AppRow({
  app,
  selected,
  onSelect,
}: {
  app: AppInfo;
  selected: boolean;
  onSelect: () => void;
}) {
  const [nameRef, nameTruncated] = useIsTruncated<HTMLSpanElement>();

  const nameSpan = (
    <span ref={nameRef} className="min-w-0 truncate">
      {app.name}
    </span>
  );

  return (
    <Item
      id={IDS.sidebarAppRow(app.path)}
      size="sm"
      className={cn(
        "relative w-full flex-nowrap rounded-none cursor-pointer hover:bg-sidebar-accent",
        selected && [
          "bg-primary/10 text-foreground",
          "before:absolute before:inset-y-0 before:left-0 before:w-0.5 before:bg-primary",
        ],
      )}
      onClick={onSelect}
    >
      <ItemContent className="min-w-0">
        <ItemTitle
          className={cn(
            "flex min-w-0 items-baseline gap-2",
            selected && "font-semibold",
          )}
        >
          {nameTruncated ? (
            <Tooltip>
              <TooltipTrigger render={nameSpan} />
              <TooltipContent side="right">{app.name}</TooltipContent>
            </Tooltip>
          ) : (
            nameSpan
          )}
          {app.version && (
            <span className="shrink-0 font-mono text-[11px] font-normal text-muted-foreground">
              {app.version}
            </span>
          )}
        </ItemTitle>
        {app.bundleId && (
          <ItemDescription className="block min-w-0 truncate font-mono text-xs">
            {app.bundleId}
          </ItemDescription>
        )}
      </ItemContent>
      {app.running && (
        <ItemActions className="shrink-0">
          <Tooltip>
            <TooltipTrigger
              render={
                <span
                  id={IDS.sidebarRunningIcon(app.path)}
                  className="inline-flex shrink-0 items-center text-destructive"
                  aria-label="Running — cannot be uninstalled"
                >
                  <LockIcon className="size-4" />
                </span>
              }
            />
            <TooltipContent>Running — cannot be uninstalled</TooltipContent>
          </Tooltip>
        </ItemActions>
      )}
    </Item>
  );
}

function SidebarFooter({ total }: { total: number }) {
  return (
    <div
      id={IDS.sidebarFooter}
      className="flex shrink-0 items-center justify-between border-t px-3 py-2 text-xs text-muted-foreground"
    >
      <span>
        {total} app{total === 1 ? "" : "s"}
      </span>
      <span className="font-mono">v{__APP_VERSION__}</span>
    </div>
  );
}
