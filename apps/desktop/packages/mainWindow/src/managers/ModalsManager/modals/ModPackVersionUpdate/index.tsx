import { Button, Dropdown } from "@gd/ui";
import { ModalProps, useModal } from "../..";
import ModalLayout from "../../ModalLayout";
import { rspc } from "@/utils/rspcClient";
import { instanceId } from "@/utils/browser";
import { Show, createSignal } from "solid-js";
import { FEInstanceId, Modpack } from "@gd/core_module/bindings";
import { useGDNavigate } from "@/managers/NavigationManager";
import { useTransContext } from "@gd/i18n";

const ModPackVersionUpdate = (props: ModalProps) => {
  const [t] = useTransContext();
  const [currentPlatform, setCurrentPlatform] = createSignal<
    "modrinth" | "curseforge" | null
  >(null);
  const [selectedVersion, setSelectedVersion] = createSignal<string>("");
  const navigate = useGDNavigate();
  const modalContext = useModal();
  const instance = rspc.createQuery(() => [
    "instance.getInstanceDetails",
    instanceId() as number
  ]);
  const changeModpackMutation = rspc.createMutation(
    ["instance.changeModpack"],
    {
      onSuccess() {
        modalContext?.closeModal();
        navigate("/library");
      }
    }
  );
  const getProjectId = () => {
    const modpack = instance.data?.modpack?.modpack;
    if (modpack) {
      if (modpack.type === "curseforge") {
        setCurrentPlatform("curseforge");
        return {
          projectId: modpack.value.project_id,
          fileId: modpack.value.file_id
        };
      } else {
        setCurrentPlatform("modrinth");
        return {
          projectId: modpack.value.project_id,
          fileId: modpack.value.version_id
        };
      }
    }

    return undefined;
  };

  const responseCF = rspc.createQuery(
    () => [
      "modplatforms.curseforge.getModFiles",
      {
        modId: getProjectId()?.projectId as number,
        query: {
          pageSize: 300
        }
      }
    ],
    {
      enabled: currentPlatform() === "curseforge"
    }
  );

  const responseModrinth = rspc.createQuery(
    () => [
      "modplatforms.modrinth.getProjectVersions",
      {
        project_id: getProjectId()?.projectId.toString()!
      }
    ],
    {
      enabled: currentPlatform() === "modrinth"
    }
  );

  const response = () =>
    currentPlatform() === "curseforge" ? responseCF : responseModrinth;

  const options = () => {
    if (currentPlatform() === "curseforge") {
      return responseCF.data?.data.map((file) => ({
        label: (
          <div class="flex justify-between w-full">
            <span>{file.displayName}</span>
            <Show when={file.id === getProjectId()?.fileId}>
              <span class="text-green-500">{`[ Current ]`}</span>
            </Show>
          </div>
        ),
        key: file.id.toString()
      }));
    } else {
      return responseModrinth.data?.map((file) => ({
        label: (
          <div class="flex justify-between w-full">
            <span>{file.name}</span>
            <Show when={file.id === getProjectId()?.fileId}>
              <span class="text-green-500">{`[ Current ]`}</span>
            </Show>
          </div>
        ),
        key: file.id.toString()
      }));
    }
  };

  const handleUpdate = () => {
    changeModpackMutation.mutate({
      instance: instanceId() as FEInstanceId,
      modpack: {
        type: currentPlatform(),
        value:
          currentPlatform() === "curseforge"
            ? {
                project_id: getProjectId()?.projectId as number,
                file_id: parseInt(selectedVersion())
              }
            : {
                project_id: getProjectId()?.projectId.toString() as string,
                version_id: selectedVersion()
              }
      } as Modpack
    });
  };

  return (
    <ModalLayout
      noHeader={props.noHeader}
      title={props?.title}
      overflowHiddenDisabled={true}
      noPadding={true}
      scrollable="overflow-y-scroll scrollbar-hide"
    >
      <div class="flex flex-col p-4 w-120 gap-4">
        <Show when={response().isLoading || instance.isLoading}>
          loading ...
        </Show>
        <Show when={!response().isLoading && !instance.isLoading}>
          <Dropdown
            class="bg-darkSlate-800 w-full"
            options={options()}
            onChange={(option) => {
              console.log(option.key);
              setSelectedVersion(option.key.toString());
            }}
          />

          <div class="flex justify-between">
            <Button
              type="outline"
              onClick={() => {
                modalContext?.closeModal();
              }}
            >
              {t("instance.cancel_export")}
            </Button>
            <Button type="primary" onClick={handleUpdate}>
              {t("instance.instance_modal_instance_update")}
            </Button>
          </div>
        </Show>
      </div>
    </ModalLayout>
  );
};

export default ModPackVersionUpdate;
