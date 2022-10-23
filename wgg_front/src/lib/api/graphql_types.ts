// THIS FILE IS GENERATED
/* eslint-disable */
import { TypedDocumentNode as DocumentNode } from '@graphql-typed-document-node/core';
export type Maybe<T> = T;
export type InputMaybe<T> = T;
export type Exact<T extends { [key: string]: unknown }> = { [K in keyof T]: T[K] };
export type MakeOptional<T, K extends keyof T> = Omit<T, K> & { [SubKey in K]?: Maybe<T[SubKey]> };
export type MakeMaybe<T, K extends keyof T> = Omit<T, K> & { [SubKey in K]: Maybe<T[SubKey]> };
/** All built-in and custom scalars, mapped to their actual values */
export type Scalars = {
  ID: string;
  String: string;
  Boolean: boolean;
  Int: number;
  Float: number;
  /**
   * Implement the DateTime<Utc> scalar
   *
   * The input/output is a string in RFC3339 format.
   */
  DateTime: Date;
};

export type AggregateCreateInput = {
  ingredients: Array<ProviderProductInput>;
  name: Scalars['String'];
};

export type AggregateCreatePayload = {
  __typename?: 'AggregateCreatePayload';
  /** The newly created aggregate ingredient */
  data: AggregateIngredient;
};

export type AggregateDeletePayload = {
  __typename?: 'AggregateDeletePayload';
  /** The amount of aggregate ingredients deleted. */
  deleted: Scalars['Int'];
};

export type AggregateIngredient = {
  __typename?: 'AggregateIngredient';
  createdAt: Scalars['DateTime'];
  id: Scalars['Int'];
  imageUrl?: Maybe<Scalars['String']>;
  /** Return all composite ingredients which are part of this aggregate ingredient. */
  ingredients: Array<WggSearchProduct>;
  name: Scalars['String'];
  /** Returns the average price of all constituent ingredients. */
  price: Scalars['Int'];
};


export type AggregateIngredientPriceArgs = {
  format: PriceFilter;
};

export type AggregateIngredientConnection = {
  __typename?: 'AggregateIngredientConnection';
  edges: Array<AggregateIngredientEdge>;
  /** A list of nodes. */
  nodes: Array<AggregateIngredient>;
  /** Information about the current page. */
  pageInfo: PageInfo;
  /** The total amount of items available in this collection */
  totalCount: Scalars['Int'];
};

/** An edge in a connection */
export type AggregateIngredientEdge = {
  __typename?: 'AggregateIngredientEdge';
  cursor: Scalars['String'];
  node: AggregateIngredient;
};

export type AggregateProductInput = {
  aggregateId: Scalars['Int'];
  quantity: Scalars['Int'];
};

export type AggregateUpdateChangeSet = {
  imageUrl?: InputMaybe<Scalars['String']>;
  ingredients?: InputMaybe<Array<ProviderProductInput>>;
  name?: InputMaybe<Scalars['String']>;
};

export type AggregateUpdatePayload = {
  __typename?: 'AggregateUpdatePayload';
  /** The updated aggregate ingredient */
  data: AggregateIngredient;
};

export type AllergyTags = {
  __typename?: 'AllergyTags';
  contains: AllergyType;
  name: Scalars['String'];
};

export enum AllergyType {
  Contains = 'CONTAINS',
  MayContain = 'MAY_CONTAIN'
}

/**
 * Represents a user that is already logged in.
 * Implements [axum::extract::FromRequest] and can therefore be requested in HTTP service methods.
 */
export type AuthContext = {
  __typename?: 'AuthContext';
  /** Return all carts owned by the given user */
  carts: UserCartConnection;
  /** Return the current cart in use by this user */
  currentCart: UserCart;
  email: Scalars['String'];
  id: Scalars['Int'];
  isAdmin: Scalars['Boolean'];
  username: Scalars['String'];
};


/**
 * Represents a user that is already logged in.
 * Implements [axum::extract::FromRequest] and can therefore be requested in HTTP service methods.
 */
export type AuthContextCartsArgs = {
  after?: InputMaybe<Scalars['String']>;
  filters?: InputMaybe<CartListFilter>;
  first?: InputMaybe<Scalars['Int']>;
};

export type CartAddProductInput = {
  aggregate?: InputMaybe<AggregateProductInput>;
  notes?: InputMaybe<NoteProductInput>;
  rawProduct?: InputMaybe<RawProductInput>;
};

export type CartAddProductPayload = {
  __typename?: 'CartAddProductPayload';
  /** The current cart */
  data: UserCart;
};

export type CartAggregateProduct = CartContent & {
  __typename?: 'CartAggregateProduct';
  /**
   * Return the primary aggregate product associated with this entry
   *
   * # Accessible by
   *
   * Everyone.
   */
  aggregate: AggregateIngredient;
  createdAt: Scalars['DateTime'];
  id: Scalars['Int'];
  quantity: Scalars['Int'];
};

export type CartCompleteInput = {
  pickedProvider: Provider;
};

export type CartCompletePayload = {
  __typename?: 'CartCompletePayload';
  /** The completed cart */
  data: UserCart;
};

export type CartContent = {
  createdAt: Scalars['DateTime'];
  id: Scalars['Int'];
  quantity: Scalars['Int'];
};

export type CartListFilter = {
  and?: InputMaybe<Array<CartListFilter>>;
  /** Whether the cart has been resolved (aka completed) */
  isCompleted?: InputMaybe<Scalars['Boolean']>;
  or?: InputMaybe<Array<CartListFilter>>;
  /** The user id who owns a given cart. */
  ownedBy?: InputMaybe<Scalars['Int']>;
};

export type CartNoteProduct = CartContent & {
  __typename?: 'CartNoteProduct';
  createdAt: Scalars['DateTime'];
  id: Scalars['Int'];
  note: Scalars['String'];
  quantity: Scalars['Int'];
};

export type CartProviderProduct = CartContent & {
  __typename?: 'CartProviderProduct';
  createdAt: Scalars['DateTime'];
  id: Scalars['Int'];
  /**
   * Return the product associated with this entry
   *
   * # Accessible by
   *
   * Everyone.
   */
  product: WggSearchProduct;
  quantity: Scalars['Int'];
};

export type CartRemoveProductInput = {
  /** The aggregate id. */
  aggregate?: InputMaybe<Scalars['Int']>;
  /** The note id. */
  notes?: InputMaybe<Scalars['Int']>;
  /** The provider product id used to add this product */
  rawProduct?: InputMaybe<RemoveRawProductInput>;
};

export type CartRemoveProductPayload = {
  __typename?: 'CartRemoveProductPayload';
  /** The current cart */
  data: UserCart;
};

export type CartTally = {
  __typename?: 'CartTally';
  priceCents: Scalars['Int'];
  provider: Provider;
};

export type Description = {
  __typename?: 'Description';
  text: Scalars['String'];
  textType: TextType;
};

export type FreshLabel = {
  __typename?: 'FreshLabel';
  daysFresh: Scalars['Int'];
};

export type IngredientInfo = {
  __typename?: 'IngredientInfo';
  name: Scalars['String'];
};

export type IngredientQueryFilter = {
  and?: InputMaybe<Array<IngredientQueryFilter>>;
  /** Return all aggregate ingredients which share (part) of the given name */
  hasName?: InputMaybe<Scalars['String']>;
  or?: InputMaybe<Array<IngredientQueryFilter>>;
};

/**
 * Contains additional information relevant for an item.
 *
 * Examples include: Preparation instructions, Supplier info
 */
export type ItemInfo = {
  __typename?: 'ItemInfo';
  itemType: ItemType;
  text: Scalars['String'];
  textType: TextType;
};

export enum ItemType {
  AdditionalInfo = 'ADDITIONAL_INFO',
  CountryOfOrigin = 'COUNTRY_OF_ORIGIN',
  PreparationAdvice = 'PREPARATION_ADVICE',
  SafetyWarning = 'SAFETY_WARNING',
  StorageAdvice = 'STORAGE_ADVICE'
}

export type LoginInput = {
  /** The email of the user account */
  email: Scalars['String'];
  /** The account's password */
  password: Scalars['String'];
};

export type MoreButton = {
  __typename?: 'MoreButton';
  images: Array<Scalars['String']>;
};

export type MutationRoot = {
  __typename?: 'MutationRoot';
  /**
   * Create a new aggregate ingredient.
   * The sub-ingredients list should have at least one ingredient inside.
   * The first in the aforementioned list's image will be used as the image for the aggregate ingredient, this can later be changed.
   *
   * # Returns
   *
   * The newly created aggregate ingredient.
   *
   * # Accessible By
   *
   * Everyone.
   */
  aggregateIngredientCreate: AggregateCreatePayload;
  /**
   * Delete an aggregate ingredient.
   * All sub-ingredients referencing this ingredient will be deleted as well.
   *
   * # Accessible By
   *
   * Everyone. One can only delete aggregate ingredients owned by the current viewer, unless they're an admin.
   */
  aggregateIngredientDelete: AggregateDeletePayload;
  /**
   * Update an aggregate ingredient.
   * The sub-ingredients list should have at least one ingredient inside.
   *
   * # Returns
   *
   * The newly updated aggregate ingredient.
   *
   * # Accessible By
   *
   * Everyone. One can only update aggregate ingredients owned by the current viewer, unless they're an admin.
   */
  aggregateIngredientUpdate: AggregateUpdatePayload;
  /**
   * Mark the current cart as completed, and create a new one.
   *
   * # Accessible By
   *
   * Everyone.
   */
  cartCurrentComplete: CartCompletePayload;
  /**
   * Remove the provided item from the current cart.
   *
   * # Accessible By
   *
   * Everyone.
   */
  cartCurrentRemoveProduct: CartRemoveProductPayload;
  /**
   * Add the provided products to the current cart.
   *
   * If one adds an item that is already in the cart then the count is set to the provided amount.
   *
   * # Accessible By
   *
   * Everyone.
   */
  cartCurrentSetProduct: CartAddProductPayload;
  /**
   * Attempt to log in as the provided user
   *
   * # Accesible By
   *
   * Everyone (also unauthenticated users)
   */
  login: UserLoginPayload;
  /** Log out with the current account */
  logout: Scalars['Int'];
  /**
   * Create a new user.
   *
   * # Returns
   *
   * The newly created user.
   *
   * # Accessible By
   *
   * Admins.
   */
  userCreate: UserCreatePayload;
  /**
   * Deletes an existing user.
   *
   * # Accessible By
   *
   * Admins.
   */
  userDelete: UserDeletePayload;
  /**
   * Update an existing user.
   *
   * # Returns
   *
   * The updated user.
   *
   * # Accessible By
   *
   * Admins, or users modifying themselves.
   */
  userUpdate: UserUpdatePayload;
};


export type MutationRootAggregateIngredientCreateArgs = {
  input: AggregateCreateInput;
};


export type MutationRootAggregateIngredientDeleteArgs = {
  ids: Array<Scalars['Int']>;
};


export type MutationRootAggregateIngredientUpdateArgs = {
  id: Scalars['Int'];
  input: AggregateUpdateChangeSet;
};


export type MutationRootCartCurrentCompleteArgs = {
  input: CartCompleteInput;
};


export type MutationRootCartCurrentRemoveProductArgs = {
  input: CartRemoveProductInput;
};


export type MutationRootCartCurrentSetProductArgs = {
  input: CartAddProductInput;
};


export type MutationRootLoginArgs = {
  input: LoginInput;
};


export type MutationRootUserCreateArgs = {
  input: UserCreateInput;
};


export type MutationRootUserDeleteArgs = {
  id: Scalars['Int'];
};


export type MutationRootUserUpdateArgs = {
  id: Scalars['Int'];
  input: UserUpdateChangeSet;
};

export type NoteProductInput = {
  content: Scalars['String'];
  /**
   * If the note already exists and you want to update it then set this Id,
   * otherwise a new note will be created based on `content`.
   */
  id?: InputMaybe<Scalars['Int']>;
  quantity: Scalars['Int'];
};

export type NumberOfServings = {
  __typename?: 'NumberOfServings';
  amount: Scalars['Int'];
};

export type NutritionalInfo = {
  __typename?: 'NutritionalInfo';
  /** For what unit (e.g, `per 100g`) these items are valid. */
  infoUnit: Scalars['String'];
  items: Array<NutritionalItem>;
};

export type NutritionalItem = {
  __typename?: 'NutritionalItem';
  name: Scalars['String'];
  subValues: Array<SubNutritionalItem>;
  value: Scalars['String'];
};

/** Information about pagination in a connection */
export type PageInfo = {
  __typename?: 'PageInfo';
  /** When paginating forwards, the cursor to continue. */
  endCursor?: Maybe<Scalars['String']>;
  /** When paginating forwards, are there more items? */
  hasNextPage: Scalars['Boolean'];
  /** When paginating backwards, are there more items? */
  hasPreviousPage: Scalars['Boolean'];
  /** When paginating backwards, the cursor to continue. */
  startCursor?: Maybe<Scalars['String']>;
};

export type PrepTime = {
  __typename?: 'PrepTime';
  timeMinutes: Scalars['Int'];
};

export enum PriceFilter {
  Average = 'AVERAGE',
  Maximum = 'MAXIMUM',
  Minimum = 'MINIMUM'
}

export type PriceInfo = {
  __typename?: 'PriceInfo';
  /** The present display price (taking into account active sales). */
  displayPrice: Scalars['Int'];
  /** The full price of an article, ignoring any sales */
  originalPrice: Scalars['Int'];
  unitPrice?: Maybe<UnitPrice>;
};

export type ProductId = PromotionProduct & {
  __typename?: 'ProductId';
  id: Scalars['String'];
};

export type PromotionProduct = {
  id: Scalars['String'];
};

export enum Provider {
  Jumbo = 'JUMBO',
  Picnic = 'PICNIC'
}

export type ProviderInfo = {
  __typename?: 'ProviderInfo';
  /** The SVG logo of the grocery store */
  logoUrl: Scalars['String'];
  /** The grocery store which provided this item. */
  provider: Provider;
};

export type ProviderProductInput = {
  id: Scalars['String'];
  provider: Provider;
};

export type QueryRoot = {
  __typename?: 'QueryRoot';
  /**
   * Returns all aggregate ingredients owned by the current user.
   *
   * # Accessible By
   *
   * Everyone.
   */
  aggregateIngredients: AggregateIngredientConnection;
  /**
   * Return the current (un-resolved) cart for the viewer.
   *
   * # Accessible By
   *
   * Everyone.
   */
  cartCurrent: UserCart;
  carts: UserCartConnection;
  proAutocomplete: Array<WggAutocomplete>;
  proProduct: WggProduct;
  proPromotions: Array<WggSaleCategory>;
  proPromotionsAll: Array<WggSaleCategory>;
  proPromotionsSublist: Array<WggSearchProduct>;
  /** Return all providers which are currently active for this server. */
  proProviders: Array<ProviderInfo>;
  proSearch: Array<WggSearchProduct>;
  proSearchAll: Array<WggSearchProduct>;
  /** Returns the current user */
  viewer: AuthContext;
};


export type QueryRootAggregateIngredientsArgs = {
  after?: InputMaybe<Scalars['String']>;
  filters?: InputMaybe<IngredientQueryFilter>;
  first?: InputMaybe<Scalars['Int']>;
};


export type QueryRootCartsArgs = {
  after?: InputMaybe<Scalars['String']>;
  filters?: InputMaybe<CartListFilter>;
  first?: InputMaybe<Scalars['Int']>;
};


export type QueryRootProAutocompleteArgs = {
  provider?: Provider;
  query: Scalars['String'];
};


export type QueryRootProProductArgs = {
  productId: Scalars['String'];
  provider?: Provider;
};


export type QueryRootProPromotionsArgs = {
  provider?: Provider;
};


export type QueryRootProPromotionsSublistArgs = {
  provider?: Provider;
  sublistId: Scalars['String'];
};


export type QueryRootProSearchArgs = {
  provider?: Provider;
  query: Scalars['String'];
};


export type QueryRootProSearchAllArgs = {
  query: Scalars['String'];
};

export type RawProductInput = {
  productId: Scalars['String'];
  provider: Provider;
  quantity: Scalars['Int'];
};

export type RemoveRawProductInput = {
  productId: Scalars['String'];
  provider: Provider;
};

/** A subtitle for a particular sale. */
export type SaleDescription = {
  __typename?: 'SaleDescription';
  text: Scalars['String'];
};

/**
 * Describes the type of sale that applies to the attached object.
 *
 * Think of "1 + 1 Free", or "50% off".
 */
export type SaleLabel = {
  __typename?: 'SaleLabel';
  text: Scalars['String'];
};

/** Until what date (inclusive) the attached sale is valid. */
export type SaleValidity = {
  __typename?: 'SaleValidity';
  validFrom: Scalars['DateTime'];
  validUntil: Scalars['DateTime'];
};

export type SubNutritionalItem = {
  __typename?: 'SubNutritionalItem';
  name: Scalars['String'];
  value: Scalars['String'];
};

export enum TextType {
  Markdown = 'MARKDOWN',
  PlainText = 'PLAIN_TEXT'
}

/** If the item is unavailable */
export type UnavailableItem = {
  __typename?: 'UnavailableItem';
  explanationLong?: Maybe<Scalars['String']>;
  explanationShort?: Maybe<Scalars['String']>;
  reason: UnavailableReason;
  /**
   * Lists replacements if the store has suggested any.
   *
   * Some stores won't support this functionality, and this would therefore remain empty.
   */
  replacements: Array<WggSearchProduct>;
};

export enum UnavailableReason {
  OutOfAssortment = 'OUT_OF_ASSORTMENT',
  OutOfSeason = 'OUT_OF_SEASON',
  TemporarilyUnavailable = 'TEMPORARILY_UNAVAILABLE',
  Unknown = 'UNKNOWN'
}

export enum Unit {
  Gram = 'GRAM',
  KiloGram = 'KILO_GRAM',
  Liter = 'LITER',
  MilliLiter = 'MILLI_LITER',
  Piece = 'PIECE'
}

export type UnitPrice = {
  __typename?: 'UnitPrice';
  price: Scalars['Int'];
  unit: Unit;
};

export type UnitQuantity = {
  __typename?: 'UnitQuantity';
  amount: Scalars['Float'];
  unit: Unit;
};

export type UserCart = {
  __typename?: 'UserCart';
  /** When a cart has been *resolved*, then it is marked as completed. */
  completed: Scalars['Boolean'];
  /** When a cart has been *resolved*, then it is marked as completed. */
  completedAt?: Maybe<Scalars['DateTime']>;
  /**
   * Return all the contents of the current cart, notes, products, and aggregates.
   *
   * The contents are sorted by the timestamp they were added (recent on top)
   */
  contents: Array<CartContent>;
  id: Scalars['Int'];
  /**
   * Return the owner of this cart.
   *
   * # Accessible by
   *
   * Everyone. If the current cart is not owned by the current user then the current user needs to be an admin.
   */
  owner: AuthContext;
  /** When a cart has been *resolved*, then a particular provider will also have been picked for that cart. */
  pickedProvider?: Maybe<Provider>;
  /**
   * Return the current (possibly outdated!) price tallies for the providers relevant to this cart.
   * One should *resolve* the current cart in order to get the most up-to-date prices.
   *
   * Note that the tallies include provider specific products (e.g, if you only have milk from Picnic, but not Jumbo,
   * Picnic will have a higher tally)
   */
  tallies: Array<CartTally>;
};


export type UserCartTalliesArgs = {
  forceCurrent?: InputMaybe<Scalars['Boolean']>;
};

export type UserCartConnection = {
  __typename?: 'UserCartConnection';
  edges: Array<UserCartEdge>;
  /** A list of nodes. */
  nodes: Array<UserCart>;
  /** Information about the current page. */
  pageInfo: PageInfo;
  /** The total amount of items available in this collection */
  totalCount: Scalars['Int'];
};

/** An edge in a connection */
export type UserCartEdge = {
  __typename?: 'UserCartEdge';
  cursor: Scalars['String'];
  node: UserCart;
};

export type UserCreateInput = {
  /** The email of the user account */
  email: Scalars['String'];
  isAdmin: Scalars['Boolean'];
  /** The account's password */
  password: Scalars['String'];
  username: Scalars['String'];
};

export type UserCreatePayload = {
  __typename?: 'UserCreatePayload';
  /** The newly created user. */
  user: AuthContext;
};

export type UserDeletePayload = {
  __typename?: 'UserDeletePayload';
  /** The Id of the deleted user */
  id: Scalars['Int'];
};

export type UserLoginPayload = {
  __typename?: 'UserLoginPayload';
  /** The newly logged-in user. */
  user: AuthContext;
};

export type UserUpdateChangeSet = {
  email?: InputMaybe<Scalars['String']>;
  password?: InputMaybe<Scalars['String']>;
  username?: InputMaybe<Scalars['String']>;
};

export type UserUpdatePayload = {
  __typename?: 'UserUpdatePayload';
  /** The newly updated user. */
  user: AuthContext;
};

export type WggAutocomplete = {
  __typename?: 'WggAutocomplete';
  name: Scalars['String'];
};

export type WggDecorator = FreshLabel | MoreButton | NumberOfServings | PrepTime | SaleDescription | SaleLabel | SaleValidity | UnavailableItem;

export type WggProduct = {
  __typename?: 'WggProduct';
  /**
   * Denotes all optional bits of information, such as preparation instructions or supplier information.
   *
   * These can be useful to add as additional collapsable tabs in the front-end ui.
   */
  additionalItems: Array<ItemInfo>;
  /**
   * All information for allergy tags.
   *
   * Can be empty if the product has no allergens.
   */
  allergyInfo: Array<AllergyTags>;
  /**
   * A small check to see if the current item is unavailable.
   *
   * `decorators` might contains more information as to the nature of the disruption.
   */
  available: Scalars['Boolean'];
  /** All decorators describing the object in further detail. */
  decorators: Array<WggDecorator>;
  /** Full product description. */
  description: Description;
  /**
   * This service's ID for the current product.
   * Not transferable between [Provider]s
   */
  id: Scalars['String'];
  /** Direct URL to product image. */
  imageUrls: Array<Scalars['String']>;
  /**
   * All ingredients in a structured format.
   *
   * Can be empty for base ingredients such as cucumbers, for example.
   */
  ingredients: Array<IngredientInfo>;
  /** The name of the product. */
  name: Scalars['String'];
  /** Denotes the nutritional info, normalised to 100g. */
  nutritional?: Maybe<NutritionalInfo>;
  /** All price related information */
  priceInfo: PriceInfo;
  /** Grocery store information associated with this item */
  providerInfo: ProviderInfo;
  /** The amount of weight/liters/pieces this product represents. */
  unitQuantity: UnitQuantity;
};

export type WggSaleCategory = {
  __typename?: 'WggSaleCategory';
  decorators: Array<WggDecorator>;
  id: Scalars['String'];
  imageUrls: Array<Scalars['String']>;
  /**
   * A potentially limited selection of items, only supported for certain [Provider]s.
   *
   * Picnic is one example of such a provider.
   * Generally recommended to query for more detailed information when needed.
   */
  limitedItems: Array<PromotionProduct>;
  name: Scalars['String'];
  /** Grocery store information associated with this item */
  providerInfo: ProviderInfo;
};

export type WggSearchProduct = PromotionProduct & {
  __typename?: 'WggSearchProduct';
  /**
   * A small check to see if the current item is unavailable.
   *
   * `decorators` might contain more information as to the nature of the disruption.
   */
  available: Scalars['Boolean'];
  decorators: Array<WggDecorator>;
  /** The present display price (taking into account active sales). */
  displayPrice: Scalars['Int'];
  /** The full price of an article, ignoring any sales */
  fullPrice: Scalars['Int'];
  id: Scalars['String'];
  /** Direct URL to product image. */
  imageUrl?: Maybe<Scalars['String']>;
  name: Scalars['String'];
  /** Grocery store information associated with this item */
  providerInfo: ProviderInfo;
  unitPrice?: Maybe<UnitPrice>;
  /** The amount of weight/liters/pieces this product represents. */
  unitQuantity: UnitQuantity;
};

export type ProductCardFragment = { __typename?: 'WggSearchProduct', id: string, name: string, displayPrice: number, fullPrice: number, imageUrl?: string, available: boolean, unitQuantity: { __typename?: 'UnitQuantity', unit: Unit, amount: number }, unitPrice?: { __typename?: 'UnitPrice', unit: Unit, price: number }, decorators: Array<{ __typename: 'FreshLabel' } | { __typename: 'MoreButton' } | { __typename: 'NumberOfServings' } | { __typename: 'PrepTime' } | { __typename: 'SaleDescription' } | { __typename: 'SaleLabel', text: string } | { __typename: 'SaleValidity', validFrom: Date, validUntil: Date } | { __typename: 'UnavailableItem', reason: UnavailableReason, explanationShort?: string, explanationLong?: string, replacements: Array<{ __typename?: 'WggSearchProduct', id: string, name: string, imageUrl?: string }> }>, providerInfo: { __typename?: 'ProviderInfo', provider: Provider, logoUrl: string } };

export type CartCurrentQueryVariables = Exact<{ [key: string]: never; }>;


export type CartCurrentQuery = { __typename?: 'QueryRoot', cartCurrent: { __typename?: 'UserCart', id: number, completedAt?: Date, pickedProvider?: Provider, contents: Array<{ __typename?: 'CartAggregateProduct', id: number, quantity: number, createdAt: Date, aggregate: { __typename?: 'AggregateIngredient', id: number, name: string, imageUrl?: string, createdAt: Date, price: number, ingredients: Array<{ __typename?: 'WggSearchProduct', id: string, name: string, displayPrice: number, fullPrice: number, imageUrl?: string, available: boolean, unitQuantity: { __typename?: 'UnitQuantity', unit: Unit, amount: number }, unitPrice?: { __typename?: 'UnitPrice', unit: Unit, price: number }, decorators: Array<{ __typename: 'FreshLabel' } | { __typename: 'MoreButton' } | { __typename: 'NumberOfServings' } | { __typename: 'PrepTime' } | { __typename: 'SaleDescription' } | { __typename: 'SaleLabel', text: string } | { __typename: 'SaleValidity', validFrom: Date, validUntil: Date } | { __typename: 'UnavailableItem', reason: UnavailableReason, explanationShort?: string, explanationLong?: string, replacements: Array<{ __typename?: 'WggSearchProduct', id: string, name: string, imageUrl?: string }> }>, providerInfo: { __typename?: 'ProviderInfo', provider: Provider, logoUrl: string } }> } } | { __typename?: 'CartNoteProduct', note: string, id: number, quantity: number, createdAt: Date } | { __typename?: 'CartProviderProduct', id: number, quantity: number, createdAt: Date, product: { __typename?: 'WggSearchProduct', id: string, name: string, displayPrice: number, fullPrice: number, imageUrl?: string, available: boolean, unitQuantity: { __typename?: 'UnitQuantity', unit: Unit, amount: number }, unitPrice?: { __typename?: 'UnitPrice', unit: Unit, price: number }, decorators: Array<{ __typename: 'FreshLabel' } | { __typename: 'MoreButton' } | { __typename: 'NumberOfServings' } | { __typename: 'PrepTime' } | { __typename: 'SaleDescription' } | { __typename: 'SaleLabel', text: string } | { __typename: 'SaleValidity', validFrom: Date, validUntil: Date } | { __typename: 'UnavailableItem', reason: UnavailableReason, explanationShort?: string, explanationLong?: string, replacements: Array<{ __typename?: 'WggSearchProduct', id: string, name: string, imageUrl?: string }> }>, providerInfo: { __typename?: 'ProviderInfo', provider: Provider, logoUrl: string } } }>, tallies: Array<{ __typename?: 'CartTally', priceCents: number, provider: Provider }> } };

export type SetProductToCartMutationVariables = Exact<{
  input: CartAddProductInput;
}>;


export type SetProductToCartMutation = { __typename?: 'MutationRoot', cartCurrentSetProduct: { __typename?: 'CartAddProductPayload', data: { __typename?: 'UserCart', id: number, completedAt?: Date, pickedProvider?: Provider, contents: Array<{ __typename?: 'CartAggregateProduct', id: number, quantity: number, createdAt: Date, aggregate: { __typename?: 'AggregateIngredient', id: number, name: string, imageUrl?: string, createdAt: Date, price: number, ingredients: Array<{ __typename?: 'WggSearchProduct', id: string, name: string, displayPrice: number, fullPrice: number, imageUrl?: string, available: boolean, unitQuantity: { __typename?: 'UnitQuantity', unit: Unit, amount: number }, unitPrice?: { __typename?: 'UnitPrice', unit: Unit, price: number }, decorators: Array<{ __typename: 'FreshLabel' } | { __typename: 'MoreButton' } | { __typename: 'NumberOfServings' } | { __typename: 'PrepTime' } | { __typename: 'SaleDescription' } | { __typename: 'SaleLabel', text: string } | { __typename: 'SaleValidity', validFrom: Date, validUntil: Date } | { __typename: 'UnavailableItem', reason: UnavailableReason, explanationShort?: string, explanationLong?: string, replacements: Array<{ __typename?: 'WggSearchProduct', id: string, name: string, imageUrl?: string }> }>, providerInfo: { __typename?: 'ProviderInfo', provider: Provider, logoUrl: string } }> } } | { __typename?: 'CartNoteProduct', note: string, id: number, quantity: number, createdAt: Date } | { __typename?: 'CartProviderProduct', id: number, quantity: number, createdAt: Date, product: { __typename?: 'WggSearchProduct', id: string, name: string, displayPrice: number, fullPrice: number, imageUrl?: string, available: boolean, unitQuantity: { __typename?: 'UnitQuantity', unit: Unit, amount: number }, unitPrice?: { __typename?: 'UnitPrice', unit: Unit, price: number }, decorators: Array<{ __typename: 'FreshLabel' } | { __typename: 'MoreButton' } | { __typename: 'NumberOfServings' } | { __typename: 'PrepTime' } | { __typename: 'SaleDescription' } | { __typename: 'SaleLabel', text: string } | { __typename: 'SaleValidity', validFrom: Date, validUntil: Date } | { __typename: 'UnavailableItem', reason: UnavailableReason, explanationShort?: string, explanationLong?: string, replacements: Array<{ __typename?: 'WggSearchProduct', id: string, name: string, imageUrl?: string }> }>, providerInfo: { __typename?: 'ProviderInfo', provider: Provider, logoUrl: string } } }>, tallies: Array<{ __typename?: 'CartTally', priceCents: number, provider: Provider }> } } };

export type RemoveProductFromCartMutationVariables = Exact<{
  input: CartRemoveProductInput;
}>;


export type RemoveProductFromCartMutation = { __typename?: 'MutationRoot', cartCurrentRemoveProduct: { __typename?: 'CartRemoveProductPayload', data: { __typename?: 'UserCart', id: number, completedAt?: Date, pickedProvider?: Provider, contents: Array<{ __typename?: 'CartAggregateProduct', id: number, quantity: number, createdAt: Date, aggregate: { __typename?: 'AggregateIngredient', id: number, name: string, imageUrl?: string, createdAt: Date, price: number, ingredients: Array<{ __typename?: 'WggSearchProduct', id: string, name: string, displayPrice: number, fullPrice: number, imageUrl?: string, available: boolean, unitQuantity: { __typename?: 'UnitQuantity', unit: Unit, amount: number }, unitPrice?: { __typename?: 'UnitPrice', unit: Unit, price: number }, decorators: Array<{ __typename: 'FreshLabel' } | { __typename: 'MoreButton' } | { __typename: 'NumberOfServings' } | { __typename: 'PrepTime' } | { __typename: 'SaleDescription' } | { __typename: 'SaleLabel', text: string } | { __typename: 'SaleValidity', validFrom: Date, validUntil: Date } | { __typename: 'UnavailableItem', reason: UnavailableReason, explanationShort?: string, explanationLong?: string, replacements: Array<{ __typename?: 'WggSearchProduct', id: string, name: string, imageUrl?: string }> }>, providerInfo: { __typename?: 'ProviderInfo', provider: Provider, logoUrl: string } }> } } | { __typename?: 'CartNoteProduct', note: string, id: number, quantity: number, createdAt: Date } | { __typename?: 'CartProviderProduct', id: number, quantity: number, createdAt: Date, product: { __typename?: 'WggSearchProduct', id: string, name: string, displayPrice: number, fullPrice: number, imageUrl?: string, available: boolean, unitQuantity: { __typename?: 'UnitQuantity', unit: Unit, amount: number }, unitPrice?: { __typename?: 'UnitPrice', unit: Unit, price: number }, decorators: Array<{ __typename: 'FreshLabel' } | { __typename: 'MoreButton' } | { __typename: 'NumberOfServings' } | { __typename: 'PrepTime' } | { __typename: 'SaleDescription' } | { __typename: 'SaleLabel', text: string } | { __typename: 'SaleValidity', validFrom: Date, validUntil: Date } | { __typename: 'UnavailableItem', reason: UnavailableReason, explanationShort?: string, explanationLong?: string, replacements: Array<{ __typename?: 'WggSearchProduct', id: string, name: string, imageUrl?: string }> }>, providerInfo: { __typename?: 'ProviderInfo', provider: Provider, logoUrl: string } } }>, tallies: Array<{ __typename?: 'CartTally', priceCents: number, provider: Provider }> } } };

export type CartFragment = { __typename?: 'UserCart', id: number, completedAt?: Date, pickedProvider?: Provider, contents: Array<{ __typename?: 'CartAggregateProduct', id: number, quantity: number, createdAt: Date, aggregate: { __typename?: 'AggregateIngredient', id: number, name: string, imageUrl?: string, createdAt: Date, price: number, ingredients: Array<{ __typename?: 'WggSearchProduct', id: string, name: string, displayPrice: number, fullPrice: number, imageUrl?: string, available: boolean, unitQuantity: { __typename?: 'UnitQuantity', unit: Unit, amount: number }, unitPrice?: { __typename?: 'UnitPrice', unit: Unit, price: number }, decorators: Array<{ __typename: 'FreshLabel' } | { __typename: 'MoreButton' } | { __typename: 'NumberOfServings' } | { __typename: 'PrepTime' } | { __typename: 'SaleDescription' } | { __typename: 'SaleLabel', text: string } | { __typename: 'SaleValidity', validFrom: Date, validUntil: Date } | { __typename: 'UnavailableItem', reason: UnavailableReason, explanationShort?: string, explanationLong?: string, replacements: Array<{ __typename?: 'WggSearchProduct', id: string, name: string, imageUrl?: string }> }>, providerInfo: { __typename?: 'ProviderInfo', provider: Provider, logoUrl: string } }> } } | { __typename?: 'CartNoteProduct', note: string, id: number, quantity: number, createdAt: Date } | { __typename?: 'CartProviderProduct', id: number, quantity: number, createdAt: Date, product: { __typename?: 'WggSearchProduct', id: string, name: string, displayPrice: number, fullPrice: number, imageUrl?: string, available: boolean, unitQuantity: { __typename?: 'UnitQuantity', unit: Unit, amount: number }, unitPrice?: { __typename?: 'UnitPrice', unit: Unit, price: number }, decorators: Array<{ __typename: 'FreshLabel' } | { __typename: 'MoreButton' } | { __typename: 'NumberOfServings' } | { __typename: 'PrepTime' } | { __typename: 'SaleDescription' } | { __typename: 'SaleLabel', text: string } | { __typename: 'SaleValidity', validFrom: Date, validUntil: Date } | { __typename: 'UnavailableItem', reason: UnavailableReason, explanationShort?: string, explanationLong?: string, replacements: Array<{ __typename?: 'WggSearchProduct', id: string, name: string, imageUrl?: string }> }>, providerInfo: { __typename?: 'ProviderInfo', provider: Provider, logoUrl: string } } }>, tallies: Array<{ __typename?: 'CartTally', priceCents: number, provider: Provider }> };

type CartContentFragment_CartAggregateProduct_ = { __typename?: 'CartAggregateProduct', id: number, quantity: number, createdAt: Date, aggregate: { __typename?: 'AggregateIngredient', id: number, name: string, imageUrl?: string, createdAt: Date, price: number, ingredients: Array<{ __typename?: 'WggSearchProduct', id: string, name: string, displayPrice: number, fullPrice: number, imageUrl?: string, available: boolean, unitQuantity: { __typename?: 'UnitQuantity', unit: Unit, amount: number }, unitPrice?: { __typename?: 'UnitPrice', unit: Unit, price: number }, decorators: Array<{ __typename: 'FreshLabel' } | { __typename: 'MoreButton' } | { __typename: 'NumberOfServings' } | { __typename: 'PrepTime' } | { __typename: 'SaleDescription' } | { __typename: 'SaleLabel', text: string } | { __typename: 'SaleValidity', validFrom: Date, validUntil: Date } | { __typename: 'UnavailableItem', reason: UnavailableReason, explanationShort?: string, explanationLong?: string, replacements: Array<{ __typename?: 'WggSearchProduct', id: string, name: string, imageUrl?: string }> }>, providerInfo: { __typename?: 'ProviderInfo', provider: Provider, logoUrl: string } }> } };

type CartContentFragment_CartNoteProduct_ = { __typename?: 'CartNoteProduct', note: string, id: number, quantity: number, createdAt: Date };

type CartContentFragment_CartProviderProduct_ = { __typename?: 'CartProviderProduct', id: number, quantity: number, createdAt: Date, product: { __typename?: 'WggSearchProduct', id: string, name: string, displayPrice: number, fullPrice: number, imageUrl?: string, available: boolean, unitQuantity: { __typename?: 'UnitQuantity', unit: Unit, amount: number }, unitPrice?: { __typename?: 'UnitPrice', unit: Unit, price: number }, decorators: Array<{ __typename: 'FreshLabel' } | { __typename: 'MoreButton' } | { __typename: 'NumberOfServings' } | { __typename: 'PrepTime' } | { __typename: 'SaleDescription' } | { __typename: 'SaleLabel', text: string } | { __typename: 'SaleValidity', validFrom: Date, validUntil: Date } | { __typename: 'UnavailableItem', reason: UnavailableReason, explanationShort?: string, explanationLong?: string, replacements: Array<{ __typename?: 'WggSearchProduct', id: string, name: string, imageUrl?: string }> }>, providerInfo: { __typename?: 'ProviderInfo', provider: Provider, logoUrl: string } } };

export type CartContentFragment = CartContentFragment_CartAggregateProduct_ | CartContentFragment_CartNoteProduct_ | CartContentFragment_CartProviderProduct_;

export type LogoutMutationVariables = Exact<{ [key: string]: never; }>;


export type LogoutMutation = { __typename?: 'MutationRoot', logout: number };

export type SubmitLoginMutationVariables = Exact<{
  email: Scalars['String'];
  password: Scalars['String'];
}>;


export type SubmitLoginMutation = { __typename?: 'MutationRoot', login: { __typename?: 'UserLoginPayload', user: { __typename?: 'AuthContext', id: number, email: string, username: string, isAdmin: boolean } } };

export type ViewerInfoQueryVariables = Exact<{ [key: string]: never; }>;


export type ViewerInfoQuery = { __typename?: 'QueryRoot', viewer: { __typename?: 'AuthContext', id: number, email: string, username: string, isAdmin: boolean } };

export type ViewerContextFragment = { __typename?: 'AuthContext', id: number, email: string, username: string, isAdmin: boolean };

export type GetAllPromotionsQueryVariables = Exact<{
  search: Scalars['String'];
}>;


export type GetAllPromotionsQuery = { __typename?: 'QueryRoot', proPromotions: Array<{ __typename?: 'WggSaleCategory', id: string, name: string, limitedItems: Array<{ __typename?: 'ProductId' } | { __typename?: 'WggSearchProduct', id: string, name: string, displayPrice: number, fullPrice: number, imageUrl?: string, available: boolean, unitQuantity: { __typename?: 'UnitQuantity', unit: Unit, amount: number }, unitPrice?: { __typename?: 'UnitPrice', unit: Unit, price: number }, decorators: Array<{ __typename: 'FreshLabel' } | { __typename: 'MoreButton' } | { __typename: 'NumberOfServings' } | { __typename: 'PrepTime' } | { __typename: 'SaleDescription' } | { __typename: 'SaleLabel', text: string } | { __typename: 'SaleValidity', validFrom: Date, validUntil: Date } | { __typename: 'UnavailableItem', reason: UnavailableReason, explanationShort?: string, explanationLong?: string, replacements: Array<{ __typename?: 'WggSearchProduct', id: string, name: string, imageUrl?: string }> }>, providerInfo: { __typename?: 'ProviderInfo', provider: Provider, logoUrl: string } }> }>, proSearchAll: Array<{ __typename?: 'WggSearchProduct', id: string, name: string, displayPrice: number, fullPrice: number, imageUrl?: string, available: boolean, unitQuantity: { __typename?: 'UnitQuantity', unit: Unit, amount: number }, unitPrice?: { __typename?: 'UnitPrice', unit: Unit, price: number }, decorators: Array<{ __typename: 'FreshLabel' } | { __typename: 'MoreButton' } | { __typename: 'NumberOfServings' } | { __typename: 'PrepTime' } | { __typename: 'SaleDescription' } | { __typename: 'SaleLabel', text: string } | { __typename: 'SaleValidity', validFrom: Date, validUntil: Date } | { __typename: 'UnavailableItem', reason: UnavailableReason, explanationShort?: string, explanationLong?: string, replacements: Array<{ __typename?: 'WggSearchProduct', id: string, name: string, imageUrl?: string }> }>, providerInfo: { __typename?: 'ProviderInfo', provider: Provider, logoUrl: string } }> };

export type FullProductQueryVariables = Exact<{
  provider: Provider;
  productId: Scalars['String'];
}>;


export type FullProductQuery = { __typename?: 'QueryRoot', proProduct: { __typename?: 'WggProduct', id: string, name: string, available: boolean, imageUrls: Array<string>, description: { __typename?: 'Description', text: string, textType: TextType }, priceInfo: { __typename?: 'PriceInfo', displayPrice: number, originalPrice: number, unitPrice?: { __typename?: 'UnitPrice', unit: Unit, price: number } }, unitQuantity: { __typename?: 'UnitQuantity', unit: Unit, amount: number }, ingredients: Array<{ __typename?: 'IngredientInfo', name: string }>, nutritional?: { __typename?: 'NutritionalInfo', infoUnit: string, items: Array<{ __typename?: 'NutritionalItem', name: string, value: string, subValues: Array<{ __typename?: 'SubNutritionalItem', name: string, value: string }> }> }, allergyInfo: Array<{ __typename?: 'AllergyTags', name: string, contains: AllergyType }>, additionalItems: Array<{ __typename?: 'ItemInfo', itemType: ItemType, text: string, textType: TextType }>, decorators: Array<{ __typename: 'FreshLabel', daysFresh: number } | { __typename: 'MoreButton', images: Array<string> } | { __typename: 'NumberOfServings', amount: number } | { __typename: 'PrepTime', timeMinutes: number } | { __typename: 'SaleDescription', text: string } | { __typename: 'SaleLabel', text: string } | { __typename: 'SaleValidity', validFrom: Date, validUntil: Date } | { __typename: 'UnavailableItem', reason: UnavailableReason, explanationShort?: string, explanationLong?: string, replacements: Array<{ __typename?: 'WggSearchProduct', id: string, name: string, displayPrice: number, fullPrice: number, imageUrl?: string, available: boolean, unitQuantity: { __typename?: 'UnitQuantity', unit: Unit, amount: number }, unitPrice?: { __typename?: 'UnitPrice', unit: Unit, price: number }, decorators: Array<{ __typename: 'FreshLabel' } | { __typename: 'MoreButton' } | { __typename: 'NumberOfServings' } | { __typename: 'PrepTime' } | { __typename: 'SaleDescription' } | { __typename: 'SaleLabel', text: string } | { __typename: 'SaleValidity', validFrom: Date, validUntil: Date } | { __typename: 'UnavailableItem', reason: UnavailableReason, explanationShort?: string, explanationLong?: string, replacements: Array<{ __typename?: 'WggSearchProduct', id: string, name: string, imageUrl?: string }> }>, providerInfo: { __typename?: 'ProviderInfo', provider: Provider, logoUrl: string } }> }>, providerInfo: { __typename?: 'ProviderInfo', provider: Provider, logoUrl: string } } };

export type FullProductFragment = { __typename?: 'WggProduct', id: string, name: string, available: boolean, imageUrls: Array<string>, description: { __typename?: 'Description', text: string, textType: TextType }, priceInfo: { __typename?: 'PriceInfo', displayPrice: number, originalPrice: number, unitPrice?: { __typename?: 'UnitPrice', unit: Unit, price: number } }, unitQuantity: { __typename?: 'UnitQuantity', unit: Unit, amount: number }, ingredients: Array<{ __typename?: 'IngredientInfo', name: string }>, nutritional?: { __typename?: 'NutritionalInfo', infoUnit: string, items: Array<{ __typename?: 'NutritionalItem', name: string, value: string, subValues: Array<{ __typename?: 'SubNutritionalItem', name: string, value: string }> }> }, allergyInfo: Array<{ __typename?: 'AllergyTags', name: string, contains: AllergyType }>, additionalItems: Array<{ __typename?: 'ItemInfo', itemType: ItemType, text: string, textType: TextType }>, decorators: Array<{ __typename: 'FreshLabel', daysFresh: number } | { __typename: 'MoreButton', images: Array<string> } | { __typename: 'NumberOfServings', amount: number } | { __typename: 'PrepTime', timeMinutes: number } | { __typename: 'SaleDescription', text: string } | { __typename: 'SaleLabel', text: string } | { __typename: 'SaleValidity', validFrom: Date, validUntil: Date } | { __typename: 'UnavailableItem', reason: UnavailableReason, explanationShort?: string, explanationLong?: string, replacements: Array<{ __typename?: 'WggSearchProduct', id: string, name: string, displayPrice: number, fullPrice: number, imageUrl?: string, available: boolean, unitQuantity: { __typename?: 'UnitQuantity', unit: Unit, amount: number }, unitPrice?: { __typename?: 'UnitPrice', unit: Unit, price: number }, decorators: Array<{ __typename: 'FreshLabel' } | { __typename: 'MoreButton' } | { __typename: 'NumberOfServings' } | { __typename: 'PrepTime' } | { __typename: 'SaleDescription' } | { __typename: 'SaleLabel', text: string } | { __typename: 'SaleValidity', validFrom: Date, validUntil: Date } | { __typename: 'UnavailableItem', reason: UnavailableReason, explanationShort?: string, explanationLong?: string, replacements: Array<{ __typename?: 'WggSearchProduct', id: string, name: string, imageUrl?: string }> }>, providerInfo: { __typename?: 'ProviderInfo', provider: Provider, logoUrl: string } }> }>, providerInfo: { __typename?: 'ProviderInfo', provider: Provider, logoUrl: string } };

export const ProductCardFragmentDoc = {"kind":"Document","definitions":[{"kind":"FragmentDefinition","name":{"kind":"Name","value":"ProductCardFragment"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"WggSearchProduct"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"name"}},{"kind":"Field","name":{"kind":"Name","value":"displayPrice"}},{"kind":"Field","name":{"kind":"Name","value":"fullPrice"}},{"kind":"Field","name":{"kind":"Name","value":"imageUrl"}},{"kind":"Field","name":{"kind":"Name","value":"available"}},{"kind":"Field","name":{"kind":"Name","value":"unitQuantity"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"unit"}},{"kind":"Field","name":{"kind":"Name","value":"amount"}}]}},{"kind":"Field","name":{"kind":"Name","value":"unitPrice"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"unit"}},{"kind":"Field","name":{"kind":"Name","value":"price"}}]}},{"kind":"Field","name":{"kind":"Name","value":"decorators"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"__typename"}},{"kind":"InlineFragment","typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"SaleLabel"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"text"}}]}},{"kind":"InlineFragment","typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"SaleValidity"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"validFrom"}},{"kind":"Field","name":{"kind":"Name","value":"validUntil"}}]}},{"kind":"InlineFragment","typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"UnavailableItem"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"reason"}},{"kind":"Field","name":{"kind":"Name","value":"explanationShort"}},{"kind":"Field","name":{"kind":"Name","value":"explanationLong"}},{"kind":"Field","name":{"kind":"Name","value":"replacements"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"name"}},{"kind":"Field","name":{"kind":"Name","value":"imageUrl"}}]}}]}}]}},{"kind":"Field","name":{"kind":"Name","value":"providerInfo"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"provider"}},{"kind":"Field","name":{"kind":"Name","value":"logoUrl"}}]}}]}}]} as unknown as DocumentNode<ProductCardFragment, unknown>;
export const CartContentFragmentDoc = {"kind":"Document","definitions":[{"kind":"FragmentDefinition","name":{"kind":"Name","value":"CartContentFragment"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"CartContent"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"quantity"}},{"kind":"Field","name":{"kind":"Name","value":"createdAt"}},{"kind":"InlineFragment","typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"CartNoteProduct"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"note"}}]}},{"kind":"InlineFragment","typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"CartAggregateProduct"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"aggregate"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"name"}},{"kind":"Field","name":{"kind":"Name","value":"imageUrl"}},{"kind":"Field","name":{"kind":"Name","value":"createdAt"}},{"kind":"Field","name":{"kind":"Name","value":"price"},"arguments":[{"kind":"Argument","name":{"kind":"Name","value":"format"},"value":{"kind":"EnumValue","value":"MINIMUM"}}]},{"kind":"Field","name":{"kind":"Name","value":"ingredients"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"FragmentSpread","name":{"kind":"Name","value":"ProductCardFragment"}}]}}]}}]}},{"kind":"InlineFragment","typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"CartProviderProduct"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"product"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"FragmentSpread","name":{"kind":"Name","value":"ProductCardFragment"}}]}}]}}]}},...ProductCardFragmentDoc.definitions]} as unknown as DocumentNode<CartContentFragment, unknown>;
export const CartFragmentDoc = {"kind":"Document","definitions":[{"kind":"FragmentDefinition","name":{"kind":"Name","value":"CartFragment"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"UserCart"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"contents"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"FragmentSpread","name":{"kind":"Name","value":"CartContentFragment"}}]}},{"kind":"Field","name":{"kind":"Name","value":"tallies"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"priceCents"}},{"kind":"Field","name":{"kind":"Name","value":"provider"}}]}},{"kind":"Field","name":{"kind":"Name","value":"completedAt"}},{"kind":"Field","name":{"kind":"Name","value":"pickedProvider"}}]}},...CartContentFragmentDoc.definitions]} as unknown as DocumentNode<CartFragment, unknown>;
export const ViewerContextFragmentDoc = {"kind":"Document","definitions":[{"kind":"FragmentDefinition","name":{"kind":"Name","value":"ViewerContext"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"AuthContext"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"email"}},{"kind":"Field","name":{"kind":"Name","value":"username"}},{"kind":"Field","name":{"kind":"Name","value":"isAdmin"}}]}}]} as unknown as DocumentNode<ViewerContextFragment, unknown>;
export const FullProductFragmentDoc = {"kind":"Document","definitions":[{"kind":"FragmentDefinition","name":{"kind":"Name","value":"FullProductFragment"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"WggProduct"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"name"}},{"kind":"Field","name":{"kind":"Name","value":"description"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"text"}},{"kind":"Field","name":{"kind":"Name","value":"textType"}}]}},{"kind":"Field","name":{"kind":"Name","value":"priceInfo"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"displayPrice"}},{"kind":"Field","name":{"kind":"Name","value":"originalPrice"}},{"kind":"Field","name":{"kind":"Name","value":"unitPrice"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"unit"}},{"kind":"Field","name":{"kind":"Name","value":"price"}}]}}]}},{"kind":"Field","name":{"kind":"Name","value":"unitQuantity"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"unit"}},{"kind":"Field","name":{"kind":"Name","value":"amount"}}]}},{"kind":"Field","name":{"kind":"Name","value":"available"}},{"kind":"Field","name":{"kind":"Name","value":"imageUrls"}},{"kind":"Field","name":{"kind":"Name","value":"ingredients"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"name"}}]}},{"kind":"Field","name":{"kind":"Name","value":"nutritional"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"infoUnit"}},{"kind":"Field","name":{"kind":"Name","value":"items"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"name"}},{"kind":"Field","name":{"kind":"Name","value":"value"}},{"kind":"Field","name":{"kind":"Name","value":"subValues"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"name"}},{"kind":"Field","name":{"kind":"Name","value":"value"}}]}}]}}]}},{"kind":"Field","name":{"kind":"Name","value":"allergyInfo"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"name"}},{"kind":"Field","name":{"kind":"Name","value":"contains"}}]}},{"kind":"Field","name":{"kind":"Name","value":"additionalItems"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"itemType"}},{"kind":"Field","name":{"kind":"Name","value":"text"}},{"kind":"Field","name":{"kind":"Name","value":"textType"}}]}},{"kind":"Field","name":{"kind":"Name","value":"decorators"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"__typename"}},{"kind":"InlineFragment","typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"FreshLabel"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"daysFresh"}}]}},{"kind":"InlineFragment","typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"SaleLabel"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"text"}}]}},{"kind":"InlineFragment","typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"SaleValidity"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"validFrom"}},{"kind":"Field","name":{"kind":"Name","value":"validUntil"}}]}},{"kind":"InlineFragment","typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"SaleDescription"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"text"}}]}},{"kind":"InlineFragment","typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"UnavailableItem"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"reason"}},{"kind":"Field","name":{"kind":"Name","value":"explanationShort"}},{"kind":"Field","name":{"kind":"Name","value":"explanationLong"}},{"kind":"Field","name":{"kind":"Name","value":"replacements"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"FragmentSpread","name":{"kind":"Name","value":"ProductCardFragment"}}]}}]}},{"kind":"InlineFragment","typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"PrepTime"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"timeMinutes"}}]}},{"kind":"InlineFragment","typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"NumberOfServings"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"amount"}}]}},{"kind":"InlineFragment","typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"MoreButton"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"images"}}]}}]}},{"kind":"Field","name":{"kind":"Name","value":"providerInfo"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"provider"}},{"kind":"Field","name":{"kind":"Name","value":"logoUrl"}}]}}]}},...ProductCardFragmentDoc.definitions]} as unknown as DocumentNode<FullProductFragment, unknown>;
export const CartCurrentQueryDocument = {"kind":"Document","definitions":[{"kind":"OperationDefinition","operation":"query","name":{"kind":"Name","value":"CartCurrentQuery"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"cartCurrent"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"FragmentSpread","name":{"kind":"Name","value":"CartFragment"}}]}}]}},...CartFragmentDoc.definitions]} as unknown as DocumentNode<CartCurrentQuery, CartCurrentQueryVariables>;
export const SetProductToCartDocument = {"kind":"Document","definitions":[{"kind":"OperationDefinition","operation":"mutation","name":{"kind":"Name","value":"setProductToCart"},"variableDefinitions":[{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"input"}},"type":{"kind":"NonNullType","type":{"kind":"NamedType","name":{"kind":"Name","value":"CartAddProductInput"}}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"cartCurrentSetProduct"},"arguments":[{"kind":"Argument","name":{"kind":"Name","value":"input"},"value":{"kind":"Variable","name":{"kind":"Name","value":"input"}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"data"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"FragmentSpread","name":{"kind":"Name","value":"CartFragment"}}]}}]}}]}},...CartFragmentDoc.definitions]} as unknown as DocumentNode<SetProductToCartMutation, SetProductToCartMutationVariables>;
export const RemoveProductFromCartDocument = {"kind":"Document","definitions":[{"kind":"OperationDefinition","operation":"mutation","name":{"kind":"Name","value":"removeProductFromCart"},"variableDefinitions":[{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"input"}},"type":{"kind":"NonNullType","type":{"kind":"NamedType","name":{"kind":"Name","value":"CartRemoveProductInput"}}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"cartCurrentRemoveProduct"},"arguments":[{"kind":"Argument","name":{"kind":"Name","value":"input"},"value":{"kind":"Variable","name":{"kind":"Name","value":"input"}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"data"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"FragmentSpread","name":{"kind":"Name","value":"CartFragment"}}]}}]}}]}},...CartFragmentDoc.definitions]} as unknown as DocumentNode<RemoveProductFromCartMutation, RemoveProductFromCartMutationVariables>;
export const LogoutMutationDocument = {"kind":"Document","definitions":[{"kind":"OperationDefinition","operation":"mutation","name":{"kind":"Name","value":"LogoutMutation"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"logout"}}]}}]} as unknown as DocumentNode<LogoutMutation, LogoutMutationVariables>;
export const SubmitLoginDocument = {"kind":"Document","definitions":[{"kind":"OperationDefinition","operation":"mutation","name":{"kind":"Name","value":"submitLogin"},"variableDefinitions":[{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"email"}},"type":{"kind":"NonNullType","type":{"kind":"NamedType","name":{"kind":"Name","value":"String"}}}},{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"password"}},"type":{"kind":"NonNullType","type":{"kind":"NamedType","name":{"kind":"Name","value":"String"}}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"login"},"arguments":[{"kind":"Argument","name":{"kind":"Name","value":"input"},"value":{"kind":"ObjectValue","fields":[{"kind":"ObjectField","name":{"kind":"Name","value":"email"},"value":{"kind":"Variable","name":{"kind":"Name","value":"email"}}},{"kind":"ObjectField","name":{"kind":"Name","value":"password"},"value":{"kind":"Variable","name":{"kind":"Name","value":"password"}}}]}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"user"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"FragmentSpread","name":{"kind":"Name","value":"ViewerContext"}}]}}]}}]}},...ViewerContextFragmentDoc.definitions]} as unknown as DocumentNode<SubmitLoginMutation, SubmitLoginMutationVariables>;
export const ViewerInfoQueryDocument = {"kind":"Document","definitions":[{"kind":"OperationDefinition","operation":"query","name":{"kind":"Name","value":"ViewerInfoQuery"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"viewer"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"FragmentSpread","name":{"kind":"Name","value":"ViewerContext"}}]}}]}},...ViewerContextFragmentDoc.definitions]} as unknown as DocumentNode<ViewerInfoQuery, ViewerInfoQueryVariables>;
export const GetAllPromotionsDocument = {"kind":"Document","definitions":[{"kind":"OperationDefinition","operation":"query","name":{"kind":"Name","value":"getAllPromotions"},"variableDefinitions":[{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"search"}},"type":{"kind":"NonNullType","type":{"kind":"NamedType","name":{"kind":"Name","value":"String"}}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"proPromotions"},"arguments":[{"kind":"Argument","name":{"kind":"Name","value":"provider"},"value":{"kind":"EnumValue","value":"PICNIC"}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"name"}},{"kind":"Field","name":{"kind":"Name","value":"limitedItems"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"InlineFragment","typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"WggSearchProduct"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"FragmentSpread","name":{"kind":"Name","value":"ProductCardFragment"}}]}}]}}]}},{"kind":"Field","name":{"kind":"Name","value":"proSearchAll"},"arguments":[{"kind":"Argument","name":{"kind":"Name","value":"query"},"value":{"kind":"Variable","name":{"kind":"Name","value":"search"}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"FragmentSpread","name":{"kind":"Name","value":"ProductCardFragment"}}]}}]}},...ProductCardFragmentDoc.definitions]} as unknown as DocumentNode<GetAllPromotionsQuery, GetAllPromotionsQueryVariables>;
export const FullProductQueryDocument = {"kind":"Document","definitions":[{"kind":"OperationDefinition","operation":"query","name":{"kind":"Name","value":"FullProductQuery"},"variableDefinitions":[{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"provider"}},"type":{"kind":"NonNullType","type":{"kind":"NamedType","name":{"kind":"Name","value":"Provider"}}}},{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"productId"}},"type":{"kind":"NonNullType","type":{"kind":"NamedType","name":{"kind":"Name","value":"String"}}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"proProduct"},"arguments":[{"kind":"Argument","name":{"kind":"Name","value":"provider"},"value":{"kind":"Variable","name":{"kind":"Name","value":"provider"}}},{"kind":"Argument","name":{"kind":"Name","value":"productId"},"value":{"kind":"Variable","name":{"kind":"Name","value":"productId"}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"FragmentSpread","name":{"kind":"Name","value":"FullProductFragment"}}]}}]}},...FullProductFragmentDoc.definitions]} as unknown as DocumentNode<FullProductQuery, FullProductQueryVariables>;