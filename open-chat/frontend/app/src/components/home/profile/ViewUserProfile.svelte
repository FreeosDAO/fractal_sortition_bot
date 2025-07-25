<script lang="ts">
    import { disableChit } from "@src/stores/settings";
    import {
        AvatarSize,
        type ChatSummary,
        type CommunitySummary,
        type OpenChat,
        type PublicProfile,
        type ReadonlySet,
        type ResourceKey,
        type UserSummary,
        blockedUsersStore,
        currentUserIdStore,
        mobileWidth,
        platformModeratorStore,
        publish,
        selectedChatBlockedUsersStore,
        selectedChatMembersStore,
        selectedChatSummaryStore,
        selectedCommunityBlockedUsersStore,
        selectedCommunityMembersStore,
        selectedCommunitySummaryStore,
        setRightPanelHistory,
    } from "openchat-client";

    import { getContext, onMount } from "svelte";
    import { _ } from "svelte-i18n";
    import ClockOutline from "svelte-material-icons/ClockOutline.svelte";
    import { i18nKey } from "../../../i18n/i18n";
    import { toastStore } from "../../../stores/toast";
    import Avatar from "../../Avatar.svelte";
    import Button from "../../Button.svelte";
    import ButtonGroup from "../../ButtonGroup.svelte";
    import ModalContent from "../../ModalContent.svelte";
    import Overlay from "../../Overlay.svelte";
    import Translatable from "../../Translatable.svelte";
    import Markdown from "../Markdown.svelte";
    import Badges from "./Badges.svelte";
    import ChitBalance from "./ChitBalance.svelte";
    import RoleIcon from "./RoleIcon.svelte";
    import WithRole from "./WithRole.svelte";

    const client = getContext<OpenChat>("client");

    interface Props {
        userId: string;
        alignTo?: DOMRect | undefined;
        chatButton?: boolean;
        inGlobalContext?: boolean;
        onOpenDirectChat: () => void;
        onClose: () => void;
    }

    let {
        userId,
        alignTo = undefined,
        chatButton = true,
        inGlobalContext = false,
        onOpenDirectChat,
        onClose,
    }: Props = $props();

    let profile: PublicProfile | undefined = $state();
    let user: UserSummary | undefined = $state();
    let lastOnline: number | undefined = $state();

    onMount(async () => {
        try {
            const task1 = client.getPublicProfile(userId);
            const task2 = client.getUser(userId);
            lastOnline = await client.getLastOnlineDate(userId, Date.now());
            user = await task2;
            profile = await task1;
            if (profile === undefined) {
                onClose();
            }
        } catch (e: any) {
            client.logError("Failed to load user profile", e);
            onClose();
        }
    });

    function afterBlock(result: boolean, success: ResourceKey, failure: ResourceKey) {
        if (!result) {
            toastStore.showFailureToast(failure);
        } else {
            toastStore.showSuccessToast(success);
        }
    }

    function blockUser() {
        if ($selectedChatSummaryStore !== undefined) {
            if ($selectedChatSummaryStore.kind === "direct_chat") {
                client.blockUserFromDirectChat(userId).then((success) => {
                    afterBlock(success, i18nKey("blockUserSucceeded"), i18nKey("blockUserFailed"));
                });
                onClose();
                return;
            }
            if ($selectedChatSummaryStore.kind === "group_chat") {
                client.blockUser($selectedChatSummaryStore.id, userId).then((success) => {
                    afterBlock(success, i18nKey("blockUserSucceeded"), i18nKey("blockUserFailed"));
                });
                onClose();
                return;
            }
        }
        if ($selectedCommunitySummaryStore !== undefined) {
            client
                .blockCommunityUser($selectedCommunitySummaryStore.id, userId)
                .then((success) =>
                    afterBlock(success, i18nKey("blockUserSucceeded"), i18nKey("blockUserFailed")),
                );
            onClose();
            return;
        }
    }

    function unblockUser() {
        if ($selectedChatSummaryStore !== undefined) {
            if ($selectedChatSummaryStore.kind === "direct_chat") {
                client.unblockUserFromDirectChat(userId).then((success) => {
                    afterBlock(
                        success,
                        i18nKey("unblockUserSucceeded"),
                        i18nKey("unblockUserFailed"),
                    );
                });
                onClose();
                return;
            }
            if ($selectedChatSummaryStore.kind === "group_chat") {
                client.unblockUser($selectedChatSummaryStore.id, userId).then((success) => {
                    afterBlock(
                        success,
                        i18nKey("unblockUserSucceeded"),
                        i18nKey("unblockUserFailed"),
                    );
                });
                onClose();
                return;
            }
        }
        if ($selectedCommunitySummaryStore !== undefined) {
            client
                .unblockCommunityUser($selectedCommunitySummaryStore.id, userId)
                .then((success) =>
                    afterBlock(
                        success,
                        i18nKey("unblockUserSucceeded"),
                        i18nKey("unblockUserFailed"),
                    ),
                );
            onClose();
            return;
        }
    }

    function canBlockUser(
        chat: ChatSummary | undefined,
        community: CommunitySummary | undefined,
        blockedUsers: ReadonlySet<string>,
        blockedChatUsers: ReadonlySet<string>,
        blockedCommunityUsers: ReadonlySet<string>,
    ) {
        if (me || inGlobalContext) return false;

        if (chat !== undefined) {
            if (chat.kind === "direct_chat") return !blockedUsers.has(userId);
            if (chat.kind === "group_chat")
                return !blockedChatUsers.has(userId) && client.canBlockUsers(chat.id);
        }
        if (community !== undefined) {
            return !blockedCommunityUsers.has(userId) && client.canBlockUsers(community.id);
        }
        return false;
    }

    function canUnblockUser(
        chat: ChatSummary | undefined,
        community: CommunitySummary | undefined,
        blockedUsers: ReadonlySet<string>,
        blockedChatUsers: ReadonlySet<string>,
        blockedCommunityUsers: ReadonlySet<string>,
    ) {
        if (me || inGlobalContext) return false;
        if (chat !== undefined) {
            if (chat.kind === "direct_chat") return blockedUsers.has(userId);
            if (chat.kind === "group_chat")
                return blockedChatUsers.has(userId) && client.canBlockUsers(chat.id);
        }
        if (community !== undefined) {
            return blockedCommunityUsers.has(userId) && client.canBlockUsers(community.id);
        }
        return false;
    }

    function handleOpenDirectChat() {
        onOpenDirectChat();
    }

    function showUserProfile() {
        setRightPanelHistory([{ kind: "user_profile" }]);
        onClose();
    }

    function onWindowResize() {
        if (!modal) {
            onClose();
        }
    }

    function formatDate(timestamp: bigint): string {
        const date = new Date(Number(timestamp));
        return date.toLocaleDateString(undefined, {
            month: "short",
            year: "numeric",
        });
    }

    function unsuspendUser() {
        client.unsuspendUser(userId).then((success) => {
            if (success) {
                toastStore.showSuccessToast(i18nKey("unsuspendedUser"));
                onClose();
            } else {
                toastStore.showFailureToast(i18nKey("failedToUnsuspendUser"));
            }
        });
    }

    function suspendUser() {
        publish("suspendUser", userId);
        onClose();
    }
    let diamondStatus = $derived(user?.diamondStatus);
    let me = $derived(userId === $currentUserIdStore);
    let isSuspended = $derived(user?.suspended ?? false);
    let modal = $derived($mobileWidth);
    let [status, online] = $derived(
        lastOnline !== undefined && lastOnline !== 0
            ? client.formatLastOnlineDate($_, Date.now(), lastOnline)
            : ["", false],
    );
    let avatarUrl = $derived(
        profile !== undefined
            ? client.buildUserAvatarUrl(
                  import.meta.env.OC_BLOB_URL_PATTERN!,
                  userId,
                  profile.avatarId,
              )
            : "/assets/unknownUserAvatar.svg",
    );
    let joined = $derived(
        profile !== undefined ? `${$_("joined")} ${formatDate(profile.created)}` : undefined,
    );
    let displayName = $derived(
        client.getDisplayName(
            {
                userId,
                username: profile?.username ?? "",
                displayName: profile?.displayName,
            },
            inGlobalContext ? undefined : $selectedCommunityMembersStore,
        ),
    );
    let canBlock = $derived(
        canBlockUser(
            $selectedChatSummaryStore,
            $selectedCommunitySummaryStore,
            $blockedUsersStore,
            $selectedChatBlockedUsersStore,
            $selectedCommunityBlockedUsersStore,
        ),
    );
    let canUnblock = $derived(
        canUnblockUser(
            $selectedChatSummaryStore,
            $selectedCommunitySummaryStore,
            $blockedUsersStore,
            $selectedChatBlockedUsersStore,
            $selectedCommunityBlockedUsersStore,
        ),
    );
</script>

<svelte:window onresize={onWindowResize} />

{#if profile !== undefined}
    <Overlay dismissible {onClose}>
        <ModalContent
            closeIcon
            fill
            square
            compactFooter
            hideFooter={!me && !chatButton && !canBlock && !canUnblock}
            fixedWidth={false}
            large={modal}
            {alignTo}
            {onClose}>
            {#snippet header()}
                <div class="header">
                    <div class="handle">
                        <div class="display_name">
                            {displayName}
                        </div>
                        <div class="name_and_badges">
                            <div class="username">
                                @{profile!.username}
                            </div>
                            <Badges
                                uniquePerson={user?.isUniquePerson}
                                {diamondStatus}
                                streak={user?.streak} />
                            {#if user !== undefined && $selectedChatSummaryStore !== undefined && $selectedChatSummaryStore.kind !== "direct_chat"}
                                <WithRole
                                    userId={user.userId}
                                    chatMembers={$selectedChatMembersStore}
                                    communityMembers={$selectedCommunityMembersStore}>
                                    {#snippet children(communityRole, chatRole)}
                                        <RoleIcon level="community" popup role={communityRole} />
                                        <RoleIcon
                                            level={$selectedChatSummaryStore?.kind === "channel"
                                                ? "channel"
                                                : "group"}
                                            popup
                                            role={chatRole} />
                                    {/snippet}
                                </WithRole>
                            {/if}
                        </div>
                    </div>
                </div>
            {/snippet}
            {#snippet body()}
                <div class="body" class:modal>
                    <div class="avatar">
                        <Avatar url={avatarUrl} {userId} size={AvatarSize.Large} />
                    </div>
                    {#if user !== undefined && !$disableChit}
                        <ChitBalance size={"small"} {me} totalEarned={user.totalChitEarned} />
                    {/if}
                    {#if profile!.bio.length > 0}
                        <p class="bio"><Markdown inline={false} text={profile!.bio} /></p>
                    {/if}
                    <div class="meta">
                        <div class="left" class:suspended={isSuspended}>
                            {#if isSuspended}
                                <Translatable resourceKey={i18nKey("accountSuspended")} />
                            {:else}
                                {#if online}
                                    <div class="online"></div>
                                {/if}
                                {status === "" ? "..." : status}
                            {/if}
                        </div>
                        <div class="right">
                            <ClockOutline size={"12px"} color={"var(--txt)"} />
                            {joined}
                        </div>
                    </div>
                </div>
            {/snippet}
            {#snippet footer()}
                <div class="footer">
                    <ButtonGroup align={"fill"}>
                        {#if chatButton && !me}
                            <Button onClick={handleOpenDirectChat} small
                                ><Translatable resourceKey={i18nKey("profile.chat")} /></Button>
                        {/if}
                        {#if me}
                            <Button onClick={showUserProfile} small
                                ><Translatable resourceKey={i18nKey("profile.settings")} /></Button>
                        {/if}
                        {#if canBlock}
                            <Button onClick={blockUser} small
                                ><Translatable resourceKey={i18nKey("profile.block")} /></Button>
                        {/if}
                        {#if canUnblock}
                            <Button onClick={unblockUser} small
                                ><Translatable resourceKey={i18nKey("profile.unblock")} /></Button>
                        {/if}
                    </ButtonGroup>
                    {#if $platformModeratorStore}
                        <div class="suspend">
                            <ButtonGroup align={"fill"}>
                                {#if isSuspended}
                                    <Button onClick={unsuspendUser} small
                                        ><Translatable
                                            resourceKey={i18nKey("unsuspendUser")} /></Button>
                                {:else}
                                    <Button onClick={suspendUser} small
                                        ><Translatable
                                            resourceKey={i18nKey("suspendUser")} /></Button>
                                {/if}
                            </ButtonGroup>
                        </div>
                    {/if}
                </div>
            {/snippet}
        </ModalContent>
    </Overlay>
{/if}

<style lang="scss">
    .body {
        position: relative;
        display: flex;
        flex-direction: column;
        @include font-size(fs-90);
        word-wrap: break-word;
        width: 320px;
        padding: 0 $sp5 0 $sp5;

        .avatar {
            padding: 0 0 $sp4 0;
            -webkit-box-reflect: below -24px linear-gradient(hsla(0, 0%, 100%, 0), hsla(
                            0,
                            0%,
                            100%,
                            0
                        )
                        45%, hsla(0, 0%, 100%, 0.2));
        }

        .bio {
            max-height: 180px;
            overflow-y: auto;
            @include font(book, normal, fs-80, 20);
            @include nice-scrollbar();
            color: var(--txt-light);
            margin-bottom: $sp3;
            width: 100%;
        }

        &.modal {
            width: 100%;
        }

        .meta {
            @include font(light, normal, fs-60);
            padding: 12px 0;
            margin-top: $sp2;
            border-top: 1px solid var(--bd);
            display: grid;
            grid-template-columns: 1fr 1fr;
            column-gap: $sp3;

            .left,
            .right {
                display: flex;
                align-items: center;
                gap: $sp2;
            }

            .left {
                justify-self: flex-start;
            }

            .right {
                justify-self: flex-end;
            }

            @include mobile() {
                .left,
                .right {
                    @include font(light, normal, fs-90);
                }
            }

            .suspended {
                color: var(--menu-warn);
            }

            .online {
                width: 10px;
                height: 10px;
                border-radius: 50%;
                background-color: green;
            }
        }
    }

    .header {
        @include font(bold, normal, fs-100, 21);
        width: 250px;

        .handle {
            overflow-wrap: anywhere;

            .username {
                font-weight: 200;
                color: var(--txt-light);
            }
        }

        .name_and_badges {
            display: inline-flex;
            gap: $sp2;
            align-items: center;
        }
    }

    .suspend {
        margin-top: $sp3;
    }
</style>
