// Verified against source files in packages/ui/src/

// login_view.ts — custom element <login-screen>, light DOM
export const LOGIN = {
  screen: 'login-screen',
  usernameInput: '#username',
  passwordInput: '#password',
  submitButton: 'button[type="submit"]',
  registerLink: '#register_instead',
  errorMessage: '#error_message',
} as const;

// register_view.ts — custom element <register-screen>, light DOM
export const REGISTER = {
  screen: 'register-screen',
  usernameInput: '#username',
  passwordInput: '#password',
  confirmInput: '#confirm',
  emailInput: '#email',
  givenNameInput: '#given_name',
  surnameInput: '#surname',
  submitButton: 'button[type="submit"]',
  loginLink: '#login_instead',
  errorMessage: '#error_message',
} as const;

// add_item_modal.ts — appended to document.body, class="modal_background add-item-modal"
export const ADD_ITEM = {
  modal: 'add-item-modal',
  titleInput: 'add-item-modal [name="title"]',
  typeSelect: 'add-item-modal [name="item_type"]',
  submitButton: 'add-item-modal [data-role="submit"]',
  cancelButton: 'add-item-modal [data-role="cancel"]',
  errorEl: 'add-item-modal [data-role="error"]',
} as const;

// item_search_modal.ts — appended to document.body, contains item-search-inline
// item-search-inline renders input[type="search"] and .item-search-inline-results
export const SEARCH = {
  modal: 'item-search-modal',
  input: 'item-search-modal input[type="search"]',
  results: 'item-search-modal .item-search-inline-results',
  resultRow: 'item-search-modal .item-search-inline-row',
  status: 'item-search-modal .item-search-inline-status',
} as const;

// item_screen.ts — custom element <item-screen>, light DOM
export const ITEM = {
  screen: 'item-screen',
  title: 'item-screen h1[contenteditable]',
  notFoundMsg: "p:text(\"That item doesn't exist.\")",
  createItBtn: "button:text('Create it?')",
} as const;

// comments_view.ts — <comments-view> rendered inside item-screen and planner screens
export const COMMENTS = {
  view: 'comments-view',
  empty: '.comments-view-empty',
  card: '.comments-view-card',
  content: '.comments-view-content',
  editTextarea: 'textarea[data-comment-edit-id]',
  draftTextarea: 'textarea[data-comments-draft]',
  postButton: '.comments-view-composer button[type="submit"]',
  editButton: '.comments-view-actions button:text("Edit")',
  deleteButton: '.comments-view-actions button:text("Delete")',
  saveButton: '.comments-view-actions button:text("Save")',
  cancelEditButton: '.comments-view-actions button:text("Cancel")',
  draftError: '.comments-view-composer .tool-error',
} as const;

// confirm_dialog.ts — appended to document.body, uses SHADOW DOM (attachShadow open)
// Playwright auto-pierces open shadow roots, so .confirm and .cancel work directly
export const CONFIRM = {
  dialog: 'confirm-dialog',
  confirmButton: '.confirm',
  cancelButton: '.cancel',
} as const;

// side_buttons.ts — SideButton custom elements inside SideButtons shell
// Each SideButton sets this.title = entry.Title
export const SIDEBAR = {
  home: 'side-button[title="Home Page"]',
  newItem: 'side-button[title="New Item"]',
  search: 'side-button[title="Search"]',
  dailyPlanner: 'side-button[title="Daily Planner"]',
  weeklyPlanner: 'side-button[title="Weekly Planner"]',
} as const;

// Top-level app shell — rendered after successful auth
export const APP_SHELL = {
  webClient: 'zealot-web-client',
} as const;
