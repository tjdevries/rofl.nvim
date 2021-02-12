/**
 * The kind of a completion entry.
 */
export namespace CompletionItemKind {
	export const Text = 1;
	export const Method = 2;
	export const Function = 3;
	export const Constructor = 4;
	export const Field = 5;
	export const Variable = 6;
	export const Class = 7;
	export const Interface = 8;
	export const Module = 9;
	export const Property = 10;
	export const Unit = 11;
	export const Value = 12;
	export const Enum = 13;
	export const Keyword = 14;
	export const Snippet = 15;
	export const Color = 16;
	export const File = 17;
	export const Reference = 18;
	export const Folder = 19;
	export const EnumMember = 20;
	export const Constant = 21;
	export const Struct = 22;
	export const Event = 23;
	export const Operator = 24;
	export const TypeParameter = 25;
}
/**
 * Represents a collection of [completion items](#CompletionItem) to be
 * presented in the editor.
 */
export interface CompletionList {
	/**
	 * This list it not complete. Further typing should result in recomputing
	 * this list.
	 */
	isIncomplete: boolean;

	/**
	 * The completion items.
	 */
	items: CompletionItem[];
}

/**
 * Defines whether the insert text in a completion item should be interpreted as
 * plain text or a snippet.
 */
export namespace InsertTextFormat {
	/**
	 * The primary text to be inserted is treated as a plain string.
	 */
	export const PlainText = 1;

	/**
	 * The primary text to be inserted is treated as a snippet.
	 *
	 * A snippet can define tab stops and placeholders with `$1`, `$2`
	 * and `${3:foo}`. `$0` defines the final tab stop, it defaults to
	 * the end of the snippet. Placeholders with equal identifiers are linked,
	 * that is typing in one will update others too.
	 */
	export const Snippet = 2;
}

export type InsertTextFormat = 1 | 2;

/**
 * Completion item tags are extra annotations that tweak the rendering of a
 * completion item.
 *
 * @since 3.15.0
 */
export namespace CompletionItemTag {
	/**
	 * Render a completion as obsolete, usually using a strike-out.
	 */
	export const Deprecated = 1;
}

export type CompletionItemTag = 1;

/**
 * A special text edit to provide an insert and a replace operation.
 *
 * @since 3.16.0
 */
export interface InsertReplaceEdit {
	/**
	 * The string to be inserted.
	 */
	newText: string;

	/**
	 * The range if the insert is requested
	 */
	insert: Range;

	/**
	 * The range if the replace is requested.
	 */
	replace: Range;
}

/**
 * How whitespace and indentation is handled during completion
 * item insertion.
 *
 * @since 3.16.0
 */
export namespace InsertTextMode {
	/**
	 * The insertion or replace strings is taken as it is. If the
	 * value is multi line the lines below the cursor will be
	 * inserted using the indentation defined in the string value.
	 * The client will not apply any kind of adjustments to the
	 * string.
	 */
	export const asIs: 1 = 1;

	/**
	 * The editor adjusts leading whitespace of new lines so that
	 * they match the indentation up to the cursor of the line for
	 * which the item is accepted.
	 *
	 * Consider a line like this: <2tabs><cursor><3tabs>foo. Accepting a
	 * multi line completion item is indented using 2 tabs and all
	 * following lines inserted will be indented using 2 tabs as well.
	 */
	export const adjustIndentation: 2 = 2;
}

export type InsertTextMode = 1 | 2;

export interface CompletionItem {
	/**
	 * The label of this completion item. By default
	 * also the text that is inserted when selecting
	 * this completion.
	 */
	label: string;

	/**
	 * The kind of this completion item. Based of the kind
	 * an icon is chosen by the editor. The standardized set
	 * of available values is defined in `CompletionItemKind`.
	 */
	kind?: CompletionItemKind;

	/**
	 * Tags for this completion item.
	 *
	 * @since 3.15.0
	 */
	tags?: CompletionItemTag[];

	/**
	 * A human-readable string with additional information
	 * about this item, like type or symbol information.
	 */
	detail?: string;

	/**
	 * A human-readable string that represents a doc-comment.
	 */
	documentation?: string | MarkupContent;

	/**
	 * Indicates if this item is deprecated.
	 *
	 * @deprecated Use `tags` instead if supported.
	 */
	deprecated?: boolean;

	/**
	 * Select this item when showing.
	 *
	 * *Note* that only one completion item can be selected and that the
	 * tool / client decides which item that is. The rule is that the *first*
	 * item of those that match best is selected.
	 */
	preselect?: boolean;

	/**
	 * A string that should be used when comparing this item
	 * with other items. When `falsy` the label is used.
	 */
	sortText?: string;

	/**
	 * A string that should be used when filtering a set of
	 * completion items. When `falsy` the label is used.
	 */
	filterText?: string;

	/**
	 * A string that should be inserted into a document when selecting
	 * this completion. When `falsy` the label is used.
	 *
	 * The `insertText` is subject to interpretation by the client side.
	 * Some tools might not take the string literally. For example
	 * VS Code when code complete is requested in this example
	 * `con<cursor position>` and a completion item with an `insertText` of
	 * `console` is provided it will only insert `sole`. Therefore it is
	 * recommended to use `textEdit` instead since it avoids additional client
	 * side interpretation.
	 */
	insertText?: string;

	/**
	 * The format of the insert text. The format applies to both the
	 * `insertText` property and the `newText` property of a provided
	 * `textEdit`. If omitted defaults to `InsertTextFormat.PlainText`.
	 */
	insertTextFormat?: InsertTextFormat;

	/**
	 * How whitespace and indentation is handled during completion
	 * item insertion. If not provided the client's default value depends on
	 * the `textDocument.completion.insertTextMode` client capability.
	 *
	 * @since 3.16.0
	 */
	insertTextMode?: InsertTextMode;

	/**
	 * An edit which is applied to a document when selecting this completion.
	 * When an edit is provided the value of `insertText` is ignored.
	 *
	 * *Note:* The range of the edit must be a single line range and it must
	 * contain the position at which completion has been requested.
	 *
	 * Most editors support two different operations when accepting a completion
	 * item. One is to insert a completion text and the other is to replace an
	 * existing text with a completion text. Since this can usually not be
	 * predetermined by a server it can report both ranges. Clients need to
	 * signal support for `InsertReplaceEdits` via the
	 * `textDocument.completion.insertReplaceSupport` client capability
	 * property.
	 *
	 * *Note 1:* The text edit's range as well as both ranges from an insert
	 * replace edit must be a [single line] and they must contain the position
	 * at which completion has been requested.
	 * *Note 2:* If an `InsertReplaceEdit` is returned the edit's insert range
	 * must be a prefix of the edit's replace range, that means it must be
	 * contained and starting at the same position.
	 *
	 * @since 3.16.0 additional type `InsertReplaceEdit`
	 */
	textEdit?: TextEdit | InsertReplaceEdit;

	/**
	 * An optional array of additional text edits that are applied when
	 * selecting this completion. Edits must not overlap (including the same
	 * insert position) with the main edit nor with themselves.
	 *
	 * Additional text edits should be used to change text unrelated to the
	 * current cursor position (for example adding an import statement at the
	 * top of the file if the completion item will insert an unqualified type).
	 */
	additionalTextEdits?: TextEdit[];

	/**
	 * An optional set of characters that when pressed while this completion is
	 * active will accept it first and then type that character. *Note* that all
	 * commit characters should have `length=1` and that superfluous characters
	 * will be ignored.
	 */
	commitCharacters?: string[];

	/**
	 * An optional command that is executed *after* inserting this completion.
	 * *Note* that additional modifications to the current document should be
	 * described with the additionalTextEdits-property.
	 */
	command?: Command;

	/**
	 * A data entry field that is preserved on a completion item between
	 * a completion and a completion resolve request.
	 */
	data?: any
}
