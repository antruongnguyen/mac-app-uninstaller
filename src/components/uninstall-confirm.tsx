import {
  AlertDialog,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
} from "@/components/ui/alert-dialog";
import { Button } from "@/components/ui/button";
import { IDS } from "@/lib/styles";
import type { AppInfo } from "@/types/models";

interface UninstallConfirmProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  app: AppInfo | null;
  selectedCount: number;
  onConfirm: () => void;
  busy: boolean;
}

export function UninstallConfirm({
  open,
  onOpenChange,
  app,
  selectedCount,
  onConfirm,
  busy,
}: UninstallConfirmProps) {
  if (!app) return null;

  return (
    <AlertDialog open={open} onOpenChange={onOpenChange}>
      <AlertDialogContent id={IDS.confirmDialog}>
        <AlertDialogHeader>
          <AlertDialogTitle>Uninstall {app.name}?</AlertDialogTitle>
          <AlertDialogDescription>
            The application bundle and {selectedCount} related item
            {selectedCount === 1 ? "" : "s"} will be moved to the Trash. System
            locations may prompt for administrator authentication.
          </AlertDialogDescription>
        </AlertDialogHeader>
        <AlertDialogFooter>
          <Button
            id={IDS.confirmCancel}
            variant="outline"
            onClick={() => onOpenChange(false)}
            disabled={busy}
          >
            Cancel
          </Button>
          <Button
            id={IDS.confirmConfirm}
            variant="destructive"
            onClick={onConfirm}
            disabled={busy}
          >
            {busy ? "Uninstalling..." : "Move to Trash"}
          </Button>
        </AlertDialogFooter>
      </AlertDialogContent>
    </AlertDialog>
  );
}
