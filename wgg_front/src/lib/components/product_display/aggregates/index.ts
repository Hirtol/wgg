import { AddProductModalFragment, AggregateCardFragment, ProductCardFragment } from '$lib/api/graphql_types';
import { triggerModal, ModalComponent, ModalSettings } from '$lib/components/modals';
import AddProductModal from './AddProductModal.svelte';
import CreateAggregateModal from './CreateAggregateModal.svelte';
import EditAggregateModal from './EditAggregateModal.svelte';

export * from './AddProductModal.svelte';
export * from './CreateAggregateModal.svelte';
export * from './EditAggregateModal.svelte';

export function triggerCreateAggregateModal(
    onResponse?: (response: { name: string } | undefined) => void,
    toFront: boolean = true
): void {
    const modalComponent: ModalComponent = {
        ref: CreateAggregateModal,
        props: {}
    };
    const modal: ModalSettings = {
        type: 'component',
        component: modalComponent,
        response: onResponse,
        title: 'Create Aggregate Product'
    };

    triggerModal(modal, toFront);
}

export function triggerEditAggregateModal(props: AggregateCardFragment, toFront: boolean = true) {
    const modalComponent: ModalComponent = {
        ref: EditAggregateModal,
        props: {
            aggregate: props
        }
    };
    const modal: ModalSettings = {
        type: 'component',
        component: modalComponent,
        title: 'Edit Aggregate Product'
    };

    triggerModal(modal, toFront);
}

export function triggerAddProductToAggregateModal(props: AddProductModalFragment, onResponse?: (response: { item: any, store: any } | undefined) => void, toFront: boolean = true) {
    const modalComponent: ModalComponent = {
        ref: AddProductModal,
        props: {
            product: props
        }
    };
    const modal: ModalSettings = {
        type: 'component',
        component: modalComponent,
        response: onResponse,
        title: 'Add Product to Aggregate'
    };

    triggerModal(modal, toFront);
}
