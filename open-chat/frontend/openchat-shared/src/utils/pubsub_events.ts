import type { Readable } from "svelte/store";
import type {
    ChannelIdentifier,
    ChatIdentifier,
    ChitEarned,
    CommunityIdentifier,
    CommunitySummary,
    DirectChatIdentifier,
    EnhancedReplyContext,
    EphemeralMessageEvent,
    EventWrapper,
    FullWebhookDetails,
    GroupChatSummary,
    Level,
    Message,
    MessageContext,
    MultiUserChat,
    MultiUserChatIdentifier,
    Notification,
    ResourceKey,
    UpdatedRules,
    VideoCallType,
} from "..";

export type PubSubEvents = {
    startVideoCall: { chatId: ChatIdentifier; callType: VideoCallType; join: boolean };
    hangup: undefined;
    askToSpeak: undefined;
    chatWith: DirectChatIdentifier;
    showInviteGroupUsers: boolean;
    replyPrivatelyTo: EnhancedReplyContext;
    showGroupMembers: undefined;
    upgrade: undefined;
    verifyHumanity: undefined;
    deleteGroup: {
        kind: "delete";
        chatId: MultiUserChatIdentifier;
        level: Level;
        doubleCheck: { challenge: ResourceKey; response: ResourceKey };
        after?: () => void;
    };
    deleteCommunity: {
        kind: "delete_community";
        communityId: CommunityIdentifier;
        doubleCheck: { challenge: ResourceKey; response: ResourceKey };
    };
    communityDetails: CommunitySummary;
    editCommunity: CommunitySummary;
    leaveCommunity: {
        kind: "leave_community";
        communityId: CommunityIdentifier;
    };
    makeProposal: undefined;
    leaveGroup: {
        kind: "leave";
        chatId: MultiUserChatIdentifier;
        level: Level;
    };
    newGroup: undefined;
    wallet: undefined;
    profile: undefined;
    claimDailyChit: undefined;
    joinGroup: {
        group: MultiUserChat;
        select: boolean;
    };
    createCommunity: undefined;
    unarchiveChat: ChatIdentifier;
    forward: Message;
    toggleMuteNotifications: {
        chatId: ChatIdentifier;
        mute: boolean;
    };
    newChannel: boolean;
    successfulImport: ChannelIdentifier;
    showProposalFilters: undefined;
    convertGroupToCommunity: GroupChatSummary;
    clearSelection: undefined;
    editGroup: { chat: MultiUserChat; rules: UpdatedRules | undefined };
    videoCallMessageUpdated: { chatId: ChatIdentifier; messageId: bigint };
    chatUpdated: MessageContext;
    sendingMessage: MessageContext;
    sentMessage: {
        context: MessageContext;
        event: EventWrapper<Message>;
    };
    userLoggedIn: string;
    reactionSelected: { messageId: bigint; kind: "add" | "remove" };
    userSuspensionChanged: undefined;
    selectedChatInvalid: undefined;
    chitEarned: ChitEarned[];
    sendMessageFailed: boolean;
    summonWitch: undefined;
    registerBot: undefined;
    updateBot: undefined;
    removeBot: undefined;
    registerWebhook: undefined;
    updateWebhook: FullWebhookDetails;
    deleteWebhook: undefined;
    loadedMessageWindow: {
        context: MessageContext;
        messageIndex: number;
        initialLoad: boolean;
    };
    loadedNewMessages: MessageContext;
    loadedPreviousMessages: {
        context: MessageContext;
        initialLoad: boolean;
    };
    createPoll: MessageContext;
    attachGif: [MessageContext, string];
    tokenTransfer: {
        context: MessageContext;
        ledger?: string;
        amount?: bigint;
    };
    createTestMessages: [MessageContext, number];
    searchChat: string;
    remoteVideoCallStarted: {
        chatId: ChatIdentifier;
        userId: string;
        messageId: bigint;
        currentUserIsParticipant: boolean;
        callType: VideoCallType;
        timestamp: bigint;
    };
    remoteVideoCallEnded: bigint;
    notification: Notification;
    noAccess: undefined;
    notFound: undefined;
    showFailureToast: {
        resourceKey: ResourceKey | Readable<ResourceKey | undefined>;
        err?: unknown;
    };
    showSuccessToast: ResourceKey;
    ephemeralMessage: EphemeralMessageEvent;
    copyUrl: undefined;
    suspendUser: string;
};
