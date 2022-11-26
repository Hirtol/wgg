<!--
    @component
    Renders the extended description of a given product, including the collapsable accordions.
-->
<script lang="ts">
    import { FullProductFragment, ItemType, TextType } from '$lib/api/graphql_types';
    import { AccordionGroup, AccordionItem } from '@brainandbones/skeleton';
    import { Apple, Information, ListDropdown, Restaurant, ToolBox } from 'carbon-icons-svelte';
    import { marked } from 'marked';

    export let product: FullProductFragment;

    $: descrPlainText = product.description.textType != TextType.Markdown ?? true;
    $: prepAdvice = product.additionalItems.find((x) => x.itemType == ItemType.PreparationAdvice);
    $: storageAdvice = product.additionalItems.find((x) => x.itemType == ItemType.StorageAdvice);
    $: safetyWarning = product.additionalItems.find((x) => x.itemType == ItemType.SafetyWarning);
    $: additionalInfo = product.additionalItems.find((x) => x.itemType == ItemType.AdditionalInfo);
    $: countryOfOrigin = product.additionalItems.find((x) => x.itemType == ItemType.CountryOfOrigin);
</script>

<div id="description" class="max-w-prose" class:whitespace-pre-line={descrPlainText}>
    {#if !descrPlainText}
        {@html marked(product.description.text ?? '')}
    {:else}
        {product.description.text ?? ''}
    {/if}
</div>

<AccordionGroup collapse={false} spacing="">
    <!-- Preparation Advice -->
    {#if prepAdvice}
        <AccordionItem padding="py-2">
            <svelte:fragment slot="lead">
                <Restaurant size={24} title={'Preparation Advice'} />
            </svelte:fragment>

            <svelte:fragment slot="summary">
                <h3>Preparation</h3>
            </svelte:fragment>

            <svelte:fragment slot="content">
                {#if prepAdvice.textType == TextType.Markdown}
                    {@html marked(prepAdvice.text)}
                {:else}
                    {prepAdvice.text}
                {/if}
            </svelte:fragment>
        </AccordionItem>
    {/if}

    <!-- Ingredients -->
    {#if product.ingredients.length > 0}
        <AccordionItem padding="py-2">
            <svelte:fragment slot="lead">
                <ListDropdown size={24} title={'Ingredients'} />
            </svelte:fragment>

            <svelte:fragment slot="summary">
                <h3>Ingredients</h3>
            </svelte:fragment>

            <svelte:fragment slot="content">
                {product.ingredients.map((x) => x.name).join(', ')}
            </svelte:fragment>
        </AccordionItem>
    {/if}

    <!-- Nutritional Info -->
    {#if product.nutritional}
        <AccordionItem padding="py-2">
            <svelte:fragment slot="lead">
                <Apple size={24} title={'Nutritional Value'} />
            </svelte:fragment>

            <svelte:fragment slot="summary">
                <h3>Nutritional Info</h3>
            </svelte:fragment>

            <div class="table-container " slot="content">
                <table class="table-hover table w-full">
                    <thead>
                        <th />
                        <th class="text-start">{product.nutritional.infoUnit}</th>
                    </thead>
                    <tbody>
                        {#each product.nutritional.items as item}
                            <tr>
                                <td class="font-bold">{item.name}</td>
                                <td>{item.value}</td>
                            </tr>
                            {#each item.subValues as subValue}
                                <tr>
                                    <td>{subValue.name}</td>
                                    <td>{subValue.value}</td>
                                </tr>
                            {/each}
                        {/each}
                    </tbody>
                </table>
            </div>
        </AccordionItem>
    {/if}

    <!-- Storage -->
    {#if storageAdvice}
        <AccordionItem padding="py-2">
            <svelte:fragment slot="lead">
                <ToolBox size={24} title={'Storage Advice'} />
            </svelte:fragment>

            <svelte:fragment slot="summary">
                <h3>Storage Advice</h3>
            </svelte:fragment>

            <svelte:fragment slot="content">
                {#if storageAdvice.textType == TextType.Markdown}
                    {@html marked(storageAdvice.text)}
                {:else}
                    {storageAdvice.text}
                {/if}
            </svelte:fragment>
        </AccordionItem>
    {/if}

    <!-- Additional Info -->
    {#if additionalInfo || safetyWarning || countryOfOrigin}
        <AccordionItem padding="py-2">
            <svelte:fragment slot="lead">
                <Information size={24} title={'Additional information'} />
            </svelte:fragment>

            <svelte:fragment slot="summary">
                <h3>Additional information</h3>
            </svelte:fragment>

            <svelte:fragment slot="content">
                {#if additionalInfo}
                    {#if additionalInfo.textType == TextType.Markdown}
                        {@html marked(additionalInfo.text)}
                    {:else}
                        {additionalInfo.text}
                    {/if}
                {/if}
                {#if safetyWarning}
                    {#if safetyWarning.textType == TextType.Markdown}
                        {@html marked(safetyWarning.text)}
                    {:else}
                        {safetyWarning.text}
                    {/if}
                {/if}
                {#if countryOfOrigin}
                    <h6>Country of origin:</h6>
                    {#if countryOfOrigin.textType == TextType.Markdown}
                        {@html marked(countryOfOrigin.text)}
                    {:else}
                        {countryOfOrigin.text}
                    {/if}
                {/if}
            </svelte:fragment>
        </AccordionItem>
    {/if}
</AccordionGroup>
