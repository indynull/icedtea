# Dialogs

## Overview

- Use dialogs to make sure users act on information

- Two variants: basic and full-screen

- Should be dedicated to completing a single task

- Can also display information relevant to the task

- Commonly used to confirm high-risk actions like deleting progress

*Basic and full-screen dialog.*

Availability & resources

Differences from M2

- Color: New color mappings and compatibility with dynamic color
- Layout: Greater padding to account for the increased corner-radius and title size
- Position: Option for custom basic dialog positioning
- Shape: Increased corner-radius
- Typography: Larger and darker headline

*Basic dialog with rounded corner, larger headline.*

## Specs

Tokens & specs
Select a component variant below to see its elements, attributes, tokens, and their values.

Basic dialogs

*Anatomy diagram numbering dialog elements.*

Basic dialog color
Color values are implemented through design tokens. For design, this means working with color values that correspond with tokens. For implementation, a color value will be a token that references a value. Learn more about design tokens

*Color mapping diagram labeling 6 color roles across the dialog and scrim.*

Basic dialog measurements

*Annotated diagram showing padding values.*

| | Attribute| Value

| Container shape
| 28dp corner radius
| Container height
| Dynamic
| Container width
| Min 280dp; Max 560dp
| Divider height
| 1dp
| Icon size
| 24dp
| Minimum width
| 280dp 
| Maximum width
| 560dp
| Alignment with icon
| Center-aligned
| Alignment without icon
| Start-aligned
| Top/Left/right/bottom padding
| 24dp
| Padding between buttons
| 8dp
| Padding between title and body
| 16dp
| Padding between icon and title
| 16dp
| Padding between body and actions
| 24dp

Full-screen dialogs

*Diagram numbering 6 full-screen dialog elements.*

Full-screen dialog color
Color values are implemented through design tokens. For design, this means working with color values that correspond with tokens. For implementation, a color value will be a token that references a value.

*Color mapping diagram shows 5 callout markers across the dialog.*

Full-screen dialog measurements

*Diagram noting layout measurements for padding values, title, height, and action regions.*

| | Attribute| Value

| Container shape
| 0dp corner radius
| Container height
| Dynamic
| Container width
| Container width; Max 560dp
| Header height
| 56dp
| Header width
| Container width
| Headline text alignment
| Start-aligned
| Divider height
| 1dp
| Icon (close affordance) size
| 24dp
| Bottom action bar height
| 56dp
| Bottom action bar width
| Container width
| Top/left/right padding
| 24dp
| Padding between elements
| 8dp

## Guidelines

*Basic dialog in isolation*

Usage

A dialog is a modal window that appears in front of app content to provide critical information or ask for a decision. Dialogs disable all app functionality when they appear, and remain on screen until confirmed, dismissed, or a required action has been taken.
Dialogs are purposefully interruptive, so they should be used sparingly. A less disruptive alternative is to use a dropdown menu, which provides options without interrupting a user’s experience.

*Diagram of basic and full-screen dialogs.*

*Dialog in front of app content.*

*Low-priority dialog in front of app content.*

Similar components
Snackbars are also designed to show important messages.
Choose the right component based on the importance of the message. This component messaging strategy helps avoid overusing dialogs.

*Snackbar on a phone saying that new photos were synced to the device. No buttons exist.*

| | Component
| Importance
| Action needed

| Snackbar| Low importance| Optional: Snackbars may not have a button, and can disappear automatically
| Dialog| High importance| Required: Dialogs block the main content until an action is confirmed

Anatomy

Basic dialog

*Diagram of 7 elements of basic dialog.*

Full-screen dialog

*6 elements of full-screen dialog.*

Container and scrim
Dialog containers appear above other screen elements and hold the dialog’s headline, text, buttons, and list items.
To focus attention on the dialog, surfaces behind the container are scrimmed with a temporary overlay to make them less prominent.

*Basic dialog shown above a scrim overlay that reduces the prominence of the background elements.*

Headline (optional)
A dialog’s purpose should be communicated by its headline and buttons or actionable items.
Headlines should:
- Contain a brief, clear statement or question
- Avoid apologies (“Sorry for the interruption”), alarm (“Warning!”), or ambiguity (“Are you sure?”)

*Dialog title asking “Use location service?”*

*Dialog title asking “Are you sure?”*

Headlines should always be succinct. They can wrap to a second line if necessary, and be truncated.
In full-screen dialogs, long headlines or headlines of variable lengths (such as translations), can be placed in the content area instead of the app bar.

*Example full-screen dialog with truncated long headline.*

*Example full-screen dialog with short headline, and longer text in content area.*

Buttons 
Dialog actions are most often represented as buttons and allow users to confirm, dismiss, or acknowledge something.
Buttons are aligned to the trailing edge of the dialog for easier interaction. The confirmation button is always closest to the edge. 
Button alignment responds automatically for right-to-left languages, where the confirmation button is aligned to the left edge.

*Dialog with the confirmation button disabled because a required radio selection is missing.*

*Dialog with the dismissing action "Cancel" on the right of the 2 buttons.*

*Dialog with a single-action button: “OK”.*

*Dialog with 2 button choices: “Cancel”, “Got it”.*

Dialogs should contain a maximum of two actions.
- If a single action is provided, it must be an acknowledgement action
- If two actions are provided, one must be a confirming action, and the other a dismissing action

*Dialog with 2 buttons side-by-side: “Disagree”, “Agree”.*

*Dialog with 2 stacked buttons: “Turn on speed boost”, “No thanks”.*

Providing a third action, such as Learn more, is not recommended as it navigates the user away from the dialog, leaving the dialog task unfinished.
Rather than adding a third action, an inline expansion can display more information. If more extensive information is needed, provide it prior to entering the dialog.

*Dialog with 3 text buttons: Learn more, Disagree, Agree.*

Basic dialog
Basic dialogs interrupt users with urgent information, details, or actions. Common use cases for basic dialogs include alerts, quick selection, and confirmation.

*Example of basic dialog action request.*

*Example of basic dialog confirmation.*

Basic dialogs most often appear as alerts or lists, but can have a variety of layouts and component combinations, including lists, date pickers, and time pickers.

*Date picker dialog.*

*Time picker dialog.*

Full-screen dialog

Full-screen dialogs fill the entire screen, containing actions that require a series of tasks to complete. One example is creating a calendar entry with the event title, date, location, and time.
Because they take up the entire screen, full-screen dialogs are the only dialogs over which other dialogs can appear.
Use a container transform pattern to transition a FAB into a full-screen dialog.

*A FAB transitioning into a full-screen dialog.*

When a full-screen dialog is closed without being saved, a basic dialog appears in front of it to confirm selections should be discarded without saving changes.

*Closing a full-screen dialog triggers a basic dialog asking to discard the changes.*

Full-screen dialogs may be used for content or tasks that meet any of these criteria:
- Dialogs that include components which require keyboard input, such as form fields

- When changes aren’t saved instantly

- When components within the dialog open additional dialogs

Full-screen dialogs are for compact breakpoints only, like mobile devices. For medium and expanded breakpoints, use a basic dialog.

Saving selections

To save a selection in a full-screen dialog, use Save.  The close icon or dismissive action, such as Cancel or Back, should close the dialog.

Confirmation

The confirmation action should be clear about what happens next, like Send or Create. Avoid using vague terms like Done, OK, or Close. Only trigger an additional basic dialog if the action fails. Don’t disable the confirmation button.

*Full-screen dialog with create button as confirmation action.*

*Full-screen dialog with an additional basic dialog asking if you want to create this event.*

Dismissing

When someone dismisses a full-screen dialog, a basic dialog should appear to confirm that they want to discard the unsaved changes.

*A basic dialog with options to either keep editing or discard unsaved changes.*

*A full-screen dialog with a Close button as the confirming action.*

Error messages

Errors about the dialog fields should always appear inline where they occur. Some components like text fields have built-in error messaging, while others like checkboxes and radio buttons need error messages to be added next to the fields.

General errors such as network issues preventing saving or submitting should appear in a basic dialog when the confirming action fails.

Error messages should clearly but briefly explain the source of the error and how to fix it. Show all errors on the page at once so people can fix everything before trying again.

*A full-screen dialog with inline error messages for text fields.*

*A basic dialog mentioning that entries were not saved due to a connection issue.*

Dialog windows
Launching a full-screen dialog temporarily resets the app’s perceived elevation, allowing simple menus or dialogs to appear above the full-screen dialog. They cover the screen and don’t appear as a floating modal window.

Navigation
Because full-screen dialogs can only be completed, dismissed, or closed, the close “X” icon button should be the only navigation option in the app bar.

Adaptive design

Dialogs can swap variants as the breakpoint changes. For example, a full-screen dialog can change into a basic dialog at larger breakpoints.

*Example of full-screen dialog on left, simple dialog on right*

Medium breakpoint
Basic dialogs appear in a center position by default.

Their position can be overridden to provide a more ergonomic experience.

*Basic dialog on tablet photos app.*

Expanded breakpoint
Dialogs on expanded breakpoints, like desktop, are modal windows above a scrim. This puts the dialog at the forefront of a person's view, calling attention to the action prompted in the dialog.

*Example of desktop dialog.*

Basic dialogs can be custom-positioned anywhere on larger screens, respecting margins to prevent edge collision.

*Basic dialog position diagram.*

Behavior

Appearing
Dialogs appear without warning, requiring users to stop their current task. They should be used sparingly, as not every choice or setting warrants interruption.
Dialogs use an enter and exit transition pattern to appear on screen.

*Dialog entering and exiting screen using fade transition.*

Position
Dialogs retain focus until dismissed or an action has been taken, such as choosing a setting. They shouldn’t be obscured by other elements or appear partially on screen, with the exception of full-screen dialogs.

*A basic dialog covering a full-screen dialog.*

Scrolling
Most dialog content should avoid scrolling. Even when scrolling is required, the dialog title is pinned at the top, with buttons pinned at the bottom. This ensures selected content remains visible alongside the title and buttons, even upon scroll.
Dialogs don’t scroll with elements outside of the dialog, such as the background.

*Example of fixed dialog title and buttons.*

Dismissing
To close a dialog used as an alert, one of its actions must be selected.
Other dialogs may be dismissed by:
- Tapping a Cancel button, if one is shown
- Selecting an actionable list item
- Pressing the keyboard Esc key
- Tapping the scrim (Android, iOS)
- Tapping the Android system Back button
- Using another standard cancel or escape action, such as iOS VoiceOver escape gesture
If the user’s ability to dismiss a dialog is disabled, the user must choose a dialog action to proceed.

*Example of dialog alert*

When a dialog box is dismissed, the following transitions occur:
- The dialog box fades out
- The background scrim fades out

*Example of dialog box fading out*

## XR

star
Note:
This is a rapidly changing space. Guidelines are primarily intended for designers at this time. Find what’s implemented in code in the design kit.

Extended reality (XR) introduces spatial capabilities, such as using depth to make dialogs stand out from the background. Currently, spatial dialogs are only available in full space. For home space, follow Material’s general dialog guidance.

Color & elevation

XR uses color roles to communicate the elevation of UI elements. Dialogs can use two color options: surface container high or surface container highest.

star
Note:
Color and elevation for spatial dialogs can be customized by makers and are not available in Jetpack Compose yet.

*2 spatially elevated dialogs with surface-container-high and surface-container-highest color roles.*

For effective visual hierarchy, a dialog should be the most prominent element. 

Add a scrim behind a dialog to improve its visibility. Scrims prevent other content from being selected until the dialog action is complete.

*Dialog with surface-container-highest color and a scrim.*

The dialog should have the highest elevation in the product.
For example, if a dialog is surface container high, don’t use surface container highest for any other elements.

*Dialog with surface-container-high color and no scrim. An orbiter is at a higher elevation than the dialog.*

Usage

Basic dialogs are recommended when designing for XR’s expanded window sizes. This keeps the required action in the person’s field of view. Limit use of full-screen dialogs to compact window sizes, like mobile devices.

*Basic dialog in XR.*

*Full-screen dialog in XR.*

Spatial dialogs

In full space, dialogs can be elevated spatially via overrides. This helps dialogs stand out from their background in XR.

*Side view of basic dialog showcasing spatial elevation.*

Behavior

Effect
The spatial dialog should scale uniformly. It also fades in when appearing, and fades out when disappearing. 

The dialog's scrim only fades in and out.

*A direct view of a spatial dialog appearing and disappearing.*

Movement
When activated, the spatial dialog rises from the app to the highest resting level on the Z-axis. 
When the action is complete, it returns to a normal resting level.
The dialog's scrim stays at the app content level at all times. 

To prevent motion sickness, use standard easing and long duration motion tokens.

*A spatial dialog elevating on the Z-axis, as seen from a side angle.*

Placement

Consider factors like field of view, viewing distance, and possible interactions when deciding where to place dialogs in XR.

Elevation: highest resting level
Display spatial dialogs at the highest resting level. When setting the depth value of the highest resting level, make sure the elevated dialog is at a comfortable viewing distance from the person. More on spatial elevation

*An animated side view of a dialog moving from the lowest to the highest resting level.*

Center spatial dialogs in field of view
Spatial dialogs should be centered in a person’s field of view. If the dialog can't track head movements, position it in the center of the app’s content. 
If the dialog can track head movements, configure it with a lazy follow behavior. This keeps the dialog anchored to the center of a person’s field of view until an action is taken.

*A dialog follows a person’s head movements, remaining centered in their field of view.*

Accessibility considerations

XR accessibility guidelines are still evolving. Spatial dialogs should follow applicable Material dialog accessibility standards.

## Accessibility

Use cases

People should be able to use assistive technology to:
- Open and close a dialog
- Provide and submit other inputs if the dialog is interactive, such as a text field or selectable list
- Scroll the dialog to access all of its contents if that content extends beyond the container of the dialog

Interaction & style

Use sparingly
Dialogs are purposefully interruptive. This means they appear in front of app content and disrupt the flow of content for people who may, for example, be using a screen reader to navigate the page.
As such, dialogs should be used sparingly and only to provide critical information. Less critical information should be presented in a non-blocking way within the flow of app content.

*An inline tooltip doesn’t block a photo app’s content on a mobile screen.
A modal dialog blocks the content of a photo app on a mobile screen.*

*A modal dialog blocks the content of a photo app on a mobile screen.
A modal dialog blocks the content of a photo app on a mobile screen.*

200% text size
Avoid excessive text wrapping or truncation by choosing concise strings. 
On Android, headlines should be kept concise enough to fit within four lines after the text size is increased to 200%. If a headline exceeds this limit and gets truncated, provide an alternative way to access the full content in a single tap.

*A dialog with 200% text wraps multiple times in the header and description. It covers most of the mobile screen.*

Elements within dialogs
Because dialogs can contain various elements within them, refer to the relevant accessibility guidelines for each element. 

Some common examples include:
- Text fields
- Typography
- Buttons

*3 elements of a full-screen dialog.*

Initial focus

When a dialog appears, focus should automatically land on the first interactive element within the dialog.

*A modal dialog titled “Permanently delete?” whose second interactive element is focused by selecting the Tab key.*

*A modal dialog titled “Permanently delete?” whose previous interactive element is focused on by selecting both the Shift and Tab keys.*

Keyboard navigation

| | Keys| Actions

| Tab| Focus lands on the next interactive element contained in the dialog, or the first element if focus is currently on the last element
| Shift + Tab
| Focus lands on the previous interactive element contained in the dialog, or the last element if focus is currently on the first element
| Space or Enter
| Triggers or commits the action of the focused element
| Escape
| Closes the dialog

Labeling elements

The accessibility label for a dialog is typically the same as the dialog’s title or headline.
On web, basic dialogs should have the alert dialog role.

*An alert dialog with a title “Set up traffic updates?”  Its label is “Set up traffic updates?” and its role as “Alert Dialog.”*

Components contained within the dialog, such as buttons, should be labeled according to the guidelines specific to those components.

For common examples, see:
- Buttons
- Text fields

*A full-screen dialog titled “New event” containing a “Save” button and a text field, both with their own accessibility labels.*
