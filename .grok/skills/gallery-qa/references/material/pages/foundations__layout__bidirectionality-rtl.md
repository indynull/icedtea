# Bidirectionality &amp; RTL

Design products that adapt to languages that read right-to-left (RTL)

## Tab 1

Over 2 billion people read and write in right-to-left (RTL) languages like Arabic, Hebrew, Farsi, and Urdu. Layouts should support both left-to-right (LTR) and RTL languages through mirroring and other best practices to ensure content is easy for global audiences to understand and navigate. Consider the holistic experience including global writing, localizing voice and design principles for culturally appropriate icons.
Material's components are built to support RTL, such as naming elements and tokens as "leading" and "trailing." However, extra configuration may be needed to achieve specific RTL situations.

Mirroring

When a layout is changed from LTR to RTL (or vice-versa), or flipped horizontally, it’s often called mirroring. UI elements and text that typically appear on the left in LTR aligns to the right. Reading flow starts from the top right corner, instead of the top left.
Not all elements mirror with RTL languages. For example, graphs and charts maintain a LTR directionality for Persian and Urdu.

*Layout in LTR and mirrored for RTL language.*

Text rendering

Correct text rendering is foundational for a great user experience, and it’s critical for readability and usability. Text rendering has two parts:
- Alignment: How the edges of the text box are placed alongside other elements

- Directionality: How text and other elements flow within a text box, like left-to-right or right-to-left

In RTL languages, text is usually right-aligned, and elements flow from right-to-left.
Common issues with RTL language rendering are text entry, cursor position, punctuation, phone numbers, and URLs.
Improperly rendering text in RTL languages can create cognitive overload and negatively impact user sentiment and trust.

*Text field incorrectly displaying the word order of an email address and cursor placement.*

*Dialog window incorrectly displaying word order decreasing readability.*

Icons & symbols

In RTL languages, directional UI icons, like back and forward, should be mirrored. However, in Hebrew, timelines and media controls on a page should retain left-to-right directionality.
The meaning of icons and symbols can vary significantly across cultures. For additional guidance, refer to design principles for icons.

*Back and forward icons in LTR and RTL.*

*Send and question mark icons in LTR and RTL.*

Time

Linear representations of time are often mirrored in RTL language experiences.
Linear progress indicators should move from right to left for most RTL languages, except Hebrew where it should remain LTR.
Circular representations of time remain the same.

*RTL linear progress indicator filling from right to left and circular progress indicator filling clockwise.*

Media players
Media controls for video or audio players are always LTR.

*Media player with control and progress in LTR and all other content is RTL.*

Clocks
For RTL languages, the directionality of time remains LTR, and clocks still turn clockwise. However, the AM/PM symbols for 12h clocks should be placed to the left. The 24-hour clock is often used in countries where the primary language isn’t English.
Clock icons, circular refresh icons, and progress indicators with arrows pointing clockwise shouldn’t be mirrored.

*24-hour clock in RTL.*

*12-hour clock in RTL.*

Canonical layout examples

List-detail
The list-detail layout:
- Is a single-pane at compact breakpoints, switching between list and detail views

- Divides the window into two side-by-side panes on large screens

- Is mirrored in RTL

*RTL list layout on mobile.*

Feed
Use a feed layout to arrange content elements like cards in a configurable grid for quick, convenient viewing of a large amount of content. The feed layout is mirrored in RTL.

*RTL feed layout.*

Supporting pane
Use the supporting pane layout to organize content into primary and secondary display areas. The supporting pane layout is mirrored in RTL.

*RTL supporting pane in an RTL language.*

Component examples

Badges
Change the position and alignment of badges for RTL languages.

*Small badge on the top left of a folder icon.*

*Large badge on the top left of an image icon.*

Toolbars
Toolbars provide actions related to the current page. For RTL languages, mirror the order of the tools.

*RTL floating toolbar.*

App bars
App bars are placed at the top of the screen to help people navigate through a product:
- Mirror an app bar’s layout in RTL

- Flip appropriate icons, such as arrows

*4 app bars in RTL.*

Navigation rail
The navigation rail is placed on the leading edge of the screen, on the left side for LTR, and on the right for RTL.

*Nav rail on the right side for an RTL language, and left side for LTR.*

Expanded navigation rail
Expanded navigation rails that open from the side are always placed on the leading edge of the screen, on the left for LTR languages, and on the right for RTL.

*RTL expanded navigation rail, including mirrored icons.*

Text fields
Icons in text fields are optional. Leading and trailing icons change their position based on LTR or RTL contexts.

*Text fields in RTL with leading and trailing icons.*

Chips
The leading icon of input chips can be an icon, logo, or circular image.
The trailing icon is always aligned to the end side of the container. It’s placed on the right for LTR and on the left for RTL.

*Filter chips with checkmark icons in RTL layout.*

Swipe gestures

Gestures are the ways people interact with UI elements using touch or body motion.
People can navigate horizontally between peer views like tabs and to complete actions.
RTL swiping and gestures should mirror their counterparts in LTR. If a product includes a delete icon revealed when swiped from the right for LTR languages, the same should be possible on the left for RTL languages.

*RTL list layout with swipe gesture revealing additional actions.*

On Android, predictive back allows people to swipe left or right on the screen to go back or dismiss modal components.
RTL predictive back features should mirror those found in a LTR context.

*Back swipe for RTL languages. The back swipe on a bottom sheet takes person back to previous screen of a photo feed.*
