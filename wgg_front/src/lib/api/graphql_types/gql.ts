/* eslint-disable */
import * as types from './graphql';
import { TypedDocumentNode as DocumentNode } from '@graphql-typed-document-node/core';

const documents = {
    "query getAllPromotions {\n  proSearchAll(query: \"banaan\") {\n    id\n    name\n    imageUrl\n    provider\n    displayPrice\n    unitQuantity {\n      unit\n      amount\n    }\n  }\n}": types.GetAllPromotionsDocument,
    "\n        fragment AutoCompleteFragment on WggAutocomplete {\n            name\n        }\n    ": types.AutoCompleteFragmentFragmentDoc,
};

export function graphql(source: "query getAllPromotions {\n  proSearchAll(query: \"banaan\") {\n    id\n    name\n    imageUrl\n    provider\n    displayPrice\n    unitQuantity {\n      unit\n      amount\n    }\n  }\n}"): (typeof documents)["query getAllPromotions {\n  proSearchAll(query: \"banaan\") {\n    id\n    name\n    imageUrl\n    provider\n    displayPrice\n    unitQuantity {\n      unit\n      amount\n    }\n  }\n}"];
export function graphql(source: "\n        fragment AutoCompleteFragment on WggAutocomplete {\n            name\n        }\n    "): (typeof documents)["\n        fragment AutoCompleteFragment on WggAutocomplete {\n            name\n        }\n    "];

export function graphql(source: string): unknown;
export function graphql(source: string) {
  return (documents as any)[source] ?? {};
}

export type DocumentType<TDocumentNode extends DocumentNode<any, any>> = TDocumentNode extends DocumentNode<  infer TType,  any>  ? TType  : never;