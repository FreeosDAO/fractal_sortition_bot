<script lang="ts">
    import { toastStore } from "@src/stores/toast";
    import { copyToClipboard } from "@src/utils/urls";
    import {
        emptyWebhookInstance,
        iconSize,
        mobileWidth,
        OpenChat,
        validBotComponentName,
        type FullWebhookDetails,
        type MultiUserChatIdentifier,
    } from "openchat-client";
    import { getContext } from "svelte";
    import { _ } from "svelte-i18n";
    import ContentCopy from "svelte-material-icons/ContentCopy.svelte";
    import { i18nKey } from "../../i18n/i18n";
    import Button from "../Button.svelte";
    import ButtonGroup from "../ButtonGroup.svelte";
    import EditableAvatar from "../EditableAvatar.svelte";
    import Legend from "../Legend.svelte";
    import ModalContent from "../ModalContent.svelte";
    import Translatable from "../Translatable.svelte";
    import ValidatingInput from "./ValidatingInput.svelte";

    const client = getContext<OpenChat>("client");

    interface Props {
        onClose: () => void;
        chatId: MultiUserChatIdentifier;
        mode: { kind: "register" } | { kind: "update"; webhook: FullWebhookDetails };
    }

    let { onClose, chatId, mode = $bindable({ kind: "register" }) }: Props = $props();

    let busy = $state(false);
    let busyUpdate = $state(false);
    let step = $state(mode.kind as string);

    let webhook = $state({
        original: mode.kind === "update" ? mode.webhook : emptyWebhookInstance(),
        current: mode.kind === "update" ? { ...mode.webhook } : emptyWebhookInstance(),
    });

    let nameDirty = $derived(webhook.original.name !== webhook.current.name);
    let avatarDirty = $derived(webhook.original.avatarUrl !== webhook.current.avatarUrl);
    let dirty = $derived(nameDirty || avatarDirty);
    let name_errors = $derived(validBotComponentName(webhook.current.name, 3));
    let valid = $derived(name_errors.length === 0);
    let url = $derived(client.webhookUrl(webhook.current, chatId));

    let titleKey = $derived.by(() => {
        switch (step) {
            case "update":
                return i18nKey("webhook.updateTitle", { name: webhook.current.name });
            default:
                return i18nKey("webhook.registerTitle");
        }
    });

    function register() {
        if (valid && dirty) {
            busy = true;
            client
                .registerWebhook(chatId, webhook.current.name, webhook.current.avatarUrl)
                .then((success) => {
                    if (success === undefined) {
                        toastStore.showFailureToast(i18nKey("Unable to register webhook"));
                    } else {
                        webhook.current = {
                            id: success.id,
                            name: webhook.current.name,
                            secret: success.secret,
                            avatarUrl: success.avatarUrl,
                        };
                        webhook.original = { ...webhook.current };
                        step = "registered";
                    }
                })
                .finally(() => (busy = false));
        }
    }

    function update() {
        busyUpdate = true;
        if (valid && dirty) {
            client
                .updateWebhook(
                    chatId,
                    webhook.current,
                    nameDirty ? webhook.current.name : undefined,
                    avatarDirty
                        ? webhook.current.avatarUrl !== undefined
                            ? { value: webhook.current.avatarUrl }
                            : "set_to_none"
                        : undefined,
                )
                .then((success) => {
                    if (!success) {
                        toastStore.showFailureToast(i18nKey("Unable to update webhook details"));
                    } else {
                        webhook.original = { ...webhook.current };
                    }
                })
                .finally(() => (busyUpdate = false));
        }
    }

    function regenerate() {
        busy = true;
        client
            .regenerateWebhook(chatId, webhook.original.id)
            .then((success) => {
                if (success === undefined) {
                    toastStore.showFailureToast(i18nKey("Unable to regenerate webhook"));
                } else {
                    webhook.current.secret = success;
                    webhook.original.secret = success;
                }
            })
            .finally(() => (busy = false));
    }

    function onSubmit(e: Event) {
        e.preventDefault();
    }

    function avatarSelected(detail: { url: string; data: Uint8Array }) {
        webhook.current.avatarUrl = detail.url;
    }

    function copy() {
        copyToClipboard(url ?? "").then((success) => {
            if (success) {
                toastStore.showSuccessToast(i18nKey("copiedToClipboard"));
            } else {
                toastStore.showFailureToast(
                    i18nKey("failedToCopyToClipboard", {
                        url,
                    }),
                );
            }
        });
    }
</script>

<ModalContent {onClose}>
    {#snippet header()}
        <div class="header">
            <Translatable resourceKey={titleKey}></Translatable>
        </div>
    {/snippet}
    {#snippet body()}
        <div class="body">
            <form onsubmit={onSubmit} class="webhook-form">
                <Legend label={i18nKey("webhook.avatarLabel")} />
                <div class="photo">
                    <EditableAvatar
                        overlayIcon
                        size={"medium"}
                        image={webhook.current.avatarUrl}
                        onImageSelected={avatarSelected} />
                </div>

                <Legend
                    required
                    label={i18nKey("webhook.nameLabel")}
                    rules={step !== "registered" ? i18nKey("webhook.nameRules") : undefined}
                ></Legend>
                <ValidatingInput
                    minlength={3}
                    maxlength={15}
                    disabled={step === "registered"}
                    invalid={name_errors.length > 0}
                    placeholder={i18nKey("webhook.namePlaceholder")}
                    error={name_errors}
                    bind:value={webhook.current.name}>
                </ValidatingInput>
            </form>

            {#if step !== "register" && url !== undefined}
                {#if step === "update"}
                    <ButtonGroup>
                        <Button
                            onClick={update}
                            disabled={busyUpdate || !valid || !dirty}
                            loading={busyUpdate}
                            small={!$mobileWidth}
                            tiny={$mobileWidth}>
                            <Translatable resourceKey={i18nKey("webhook.updateAction")} />
                        </Button>
                    </ButtonGroup>
                {/if}
                <hr />
                <div class="url">
                    <div class="title">
                        <Translatable resourceKey={i18nKey("webhook.urlLabel")} />
                    </div>
                    <div class="copy" title={$_("copyToClipboard")} onclick={copy}>
                        <ContentCopy size={$iconSize} color={"var(--icon-txt)"} />
                    </div>
                </div>
                {url}
            {/if}
        </div>
    {/snippet}
    {#snippet footer()}
        <div class="footer">
            <ButtonGroup>
                <Button secondary small={!$mobileWidth} tiny={$mobileWidth} onClick={onClose}>
                    <Translatable
                        resourceKey={step !== "register" && !dirty
                            ? i18nKey("close")
                            : i18nKey("cancel")} />
                </Button>
                <Button
                    onClick={step === "update" ? regenerate : step === "register" ? register : copy}
                    disabled={busy || (step === "register" && (!valid || !dirty))}
                    loading={busy}
                    small={!$mobileWidth}
                    tiny={$mobileWidth}>
                    <Translatable
                        resourceKey={step === "update"
                            ? i18nKey("webhook.regenerateAction")
                            : step === "register"
                              ? i18nKey("webhook.registerAction")
                              : i18nKey("copy")} />
                </Button>
            </ButtonGroup>
        </div>
    {/snippet}
</ModalContent>

<style lang="scss">
    .webhook-form {
        :global(.input-wrapper) {
            margin-bottom: $sp5;
        }
    }
    .url {
        display: flex;
        align-items: center;
        gap: $sp3;
        margin-bottom: $sp5;

        .title {
            @include font(bold, normal, fs-130, 29);
            @include mobile() {
                @include font(bold, normal, fs-120, 29);
            }
        }

        .copy {
            display: flex;
            cursor: pointer;
        }
    }

    hr {
        margin: $sp5 0;
        color: var(--bd);
    }

    .photo {
        max-width: toRem(100);
        margin-bottom: $sp3;
    }
</style>
