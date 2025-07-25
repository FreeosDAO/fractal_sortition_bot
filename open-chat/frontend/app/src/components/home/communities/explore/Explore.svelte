<script lang="ts">
    import type { OpenChat } from "openchat-client";
    import {
        anonUserStore,
        exploreCommunitiesFiltersStore,
        iconSize,
        identityStateStore,
        ipadWidth,
        isDiamondStore,
        mobileWidth,
        offlineStore,
        publish,
        ScreenWidth,
        screenWidth,
    } from "openchat-client";
    import { getContext, onMount, tick } from "svelte";
    import { _ } from "svelte-i18n";
    import ArrowUp from "svelte-material-icons/ArrowUp.svelte";
    import CloudOffOutline from "svelte-material-icons/CloudOffOutline.svelte";
    import Plus from "svelte-material-icons/Plus.svelte";
    import Tune from "svelte-material-icons/Tune.svelte";
    import { i18nKey } from "../../../../i18n/i18n";
    import { communitySearchState } from "../../../../stores/search.svelte";
    import Search from "../../..//Search.svelte";
    import Button from "../../../Button.svelte";
    import Fab from "../../../Fab.svelte";
    import HoverIcon from "../../../HoverIcon.svelte";
    import FancyLoader from "../../../icons/FancyLoader.svelte";
    import Translatable from "../../../Translatable.svelte";
    import CommunityCard from "./CommunityCard.svelte";
    import CommunityCardLink from "./CommunityCardLink.svelte";

    const client = getContext<OpenChat>("client");

    let searching = $state(false);
    let showFab = $state(false);
    let scrollableElement: HTMLElement | null;
    let initialised = $state(false);

    function calculatePageSize(width: ScreenWidth): number {
        // make sure we get even rows of results
        switch (width) {
            case ScreenWidth.Large:
            case ScreenWidth.ExtraLarge:
                return 30;
            default:
                return 32;
        }
    }

    function createCommunity() {
        if ($anonUserStore) {
            client.updateIdentityState({
                kind: "logging_in",
                postLogin: { kind: "create_community" },
            });
            return;
        }
        if (!$isDiamondStore) {
            publish("upgrade");
        } else {
            publish("createCommunity");
        }
    }

    function search(filters: { languages: string[]; flags: number }, reset = false) {
        searching = true;
        if (reset) {
            communitySearchState.reset();
        } else {
            communitySearchState.nextPage();
        }

        client
            .exploreCommunities(
                communitySearchState.term === "" ? undefined : communitySearchState.term,
                communitySearchState.index,
                pageSize,
                filters.flags ?? 0,
                filters.languages,
            )
            .then((results) => {
                if (results.kind === "success") {
                    if (reset) {
                        communitySearchState.results = results.matches;
                    } else {
                        communitySearchState.appendResults(results.matches);
                    }
                    communitySearchState.total = results.total;
                }
            })
            .finally(() => (searching = false));
    }

    function showFilters() {
        client.pushRightPanelHistory({ kind: "community_filters" });
    }

    onMount(() => {
        tick().then(() => {
            scrollableElement = document.getElementById("communities-wrapper");
            if (scrollableElement) {
                scrollableElement.scrollTop = communitySearchState.scrollPos;
            }
            onScroll();
        });
        return exploreCommunitiesFiltersStore.subscribe((filters) => {
            if (initialised || communitySearchState.results.length === 0) {
                search(filters, true);
            }
            initialised = true;
        });
    });

    function scrollToTop() {
        if (scrollableElement) {
            scrollableElement.scrollTop = 0;
        }
    }

    function onScroll() {
        if (scrollableElement) {
            showFab = scrollableElement.scrollTop > 500;
            communitySearchState.scrollPos = scrollableElement.scrollTop;
        }
    }
    let pageSize = $derived(calculatePageSize($screenWidth));
    let more = $derived(communitySearchState.total > communitySearchState.results.length);
    let loading = $derived(searching && communitySearchState.results.length === 0);

    $effect(() => {
        if (
            $identityStateStore.kind === "logged_in" &&
            $identityStateStore.postLogin?.kind === "create_community"
        ) {
            client.clearPostLoginState();
            tick().then(() => createCommunity());
        }
    });
</script>

<div class="explore">
    <div class="header">
        <div class="title-row">
            <div class="title">
                {#if $mobileWidth}
                    <h4><Translatable resourceKey={i18nKey("communities.exploreMobile")} /></h4>
                {:else}
                    <h4><Translatable resourceKey={i18nKey("communities.explore")} /></h4>
                {/if}
            </div>
            {#if !$ipadWidth}
                <div class="search">
                    <Search
                        fill
                        bind:searchTerm={communitySearchState.term}
                        searching={false}
                        onPerformSearch={() => search($exploreCommunitiesFiltersStore, true)}
                        placeholder={i18nKey("communities.search")} />
                </div>
                <div class="create">
                    <Button onClick={createCommunity} hollow
                        ><Translatable resourceKey={i18nKey("communities.create")} /></Button>
                </div>
            {/if}
            <div class="buttons">
                {#if $ipadWidth}
                    <HoverIcon onclick={createCommunity}>
                        <Plus size={$iconSize} color={"var(--icon-txt)"} />
                    </HoverIcon>
                {/if}

                <HoverIcon title={$_("showFilters")} onclick={showFilters}>
                    <Tune size={$iconSize} color={"var(--icon-txt)"} />
                </HoverIcon>
            </div>
        </div>
        <div class="subtitle-row">
            {#if $ipadWidth}
                <div class="search">
                    <Search
                        searching={false}
                        fill
                        bind:searchTerm={communitySearchState.term}
                        onPerformSearch={() => search($exploreCommunitiesFiltersStore, true)}
                        placeholder={i18nKey("communities.search")} />
                </div>
            {/if}
        </div>
    </div>

    <div onscroll={onScroll} id="communities-wrapper" class="communities-wrapper">
        <div
            class="communities"
            class:loading
            class:empty={communitySearchState.results.length === 0}>
            {#if loading}
                <div class="loading">
                    <FancyLoader />
                </div>
            {:else if communitySearchState.results.length === 0}
                {#if $offlineStore}
                    <div class="no-match">
                        <CloudOffOutline size={"1.8em"} color={"var(--txt-light)"} />
                        <p class="sub-header">
                            <Translatable resourceKey={i18nKey("offlineError")} />
                        </p>
                    </div>
                {:else}
                    <div class="no-match">
                        <h4 class="header">
                            <Translatable resourceKey={i18nKey("communities.noMatch")} />
                        </h4>
                        <p class="sub-header">
                            <Translatable resourceKey={i18nKey("communities.refineSearch")} />
                        </p>
                    </div>
                {/if}
            {:else}
                {#each communitySearchState.results as community (community.id.communityId)}
                    <CommunityCardLink url={`/community/${community.id.communityId}`}>
                        <CommunityCard
                            id={community.id.communityId}
                            name={community.name}
                            description={community.description}
                            avatar={community.avatar}
                            banner={community.banner}
                            memberCount={community.memberCount}
                            channelCount={community.channelCount}
                            gateConfig={community.gateConfig}
                            language={community.primaryLanguage}
                            flags={community.flags}
                            verified={community.verified} />
                    </CommunityCardLink>
                {/each}
            {/if}
        </div>
        {#if more}
            <div class="more">
                <Button
                    disabled={searching}
                    loading={searching}
                    onClick={() => search($exploreCommunitiesFiltersStore, false)}
                    ><Translatable resourceKey={i18nKey("communities.loadMore")} /></Button>
            </div>
        {/if}
    </div>
    <div class:show={showFab} class="fab">
        <Fab on:click={scrollToTop}>
            <ArrowUp size={$iconSize} color={"#fff"} />
        </Fab>
    </div>
</div>

<style lang="scss">
    .explore {
        display: flex;
        flex-direction: column;
        gap: $sp4;
        padding: $sp5;
        height: 100%;
        overflow: hidden;
        position: relative;

        @include mobile() {
            padding: $sp3;
            gap: $sp3;
        }
    }

    .header {
        .title-row {
            display: flex;
            align-items: center;
            gap: $sp4;
            margin-bottom: $sp5;

            @include size-below(lg) {
                margin-bottom: $sp3;
                justify-content: space-between;
            }

            .title {
                display: flex;
                gap: $sp3;
                align-items: center;

                h4 {
                    @include font(bold, normal, fs-160, 38);
                    flex: auto;

                    @include mobile() {
                        @include font(bold, normal, fs-140, 38);
                    }
                }
            }

            .search {
                flex: auto;
            }

            .buttons {
                display: flex;
                align-items: center;
                justify-content: flex-end;
            }
        }

        .subtitle-row {
            display: flex;
            justify-content: space-between;
            gap: $sp4;

            @include size-below(lg) {
                flex-direction: column;
            }
        }
    }

    .communities-wrapper {
        @include nice-scrollbar();
        flex: auto;
        height: 3000px;
    }

    .communities {
        display: grid;
        grid-template-columns: repeat(4, 1fr);
        grid-gap: $sp5;
        margin-bottom: $sp5;

        @include size-below(xxl) {
            grid-gap: $sp4;
        }

        @include size-below(xl) {
            grid-template-columns: repeat(3, 1fr);
        }

        @include size-below(md) {
            grid-template-columns: repeat(2, 1fr);
        }

        @include size-below(sm) {
            grid-template-columns: repeat(1, 1fr);
        }

        &.loading,
        &.empty {
            height: 100%;
            grid-template-columns: repeat(1, 1fr);
            margin-bottom: 0;
        }
    }

    .more {
        text-align: center;
    }

    $size: 200px;

    .loading {
        width: $size;
        margin: auto;
    }

    .no-match {
        .header {
            @include font(bold, normal, fs-160, 38);
        }
        .sub-header {
            @include font(book, normal, fs-100, 38);
            color: var(--txt-light);
        }
        margin: auto;
        text-align: center;
    }

    .fab {
        transition: opacity ease-in-out 300ms;
        position: absolute;
        @include z-index("fab");
        right: 20px;
        bottom: 20px;
        opacity: 0;
        pointer-events: none;

        &.show {
            opacity: 1;
            pointer-events: all;
        }
    }
</style>
