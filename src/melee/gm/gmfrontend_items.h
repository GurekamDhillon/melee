/* gmfrontend_items.h - the toolkit's data: the rows of a settings-style screen (included by gmfrontend.c and by the native tests of the Atlas table
 * walker). No libc and no game headers: u8 is unsigned char in the game too. A page is one array of FrontendItem; positional initialisers stay valid
 * because new fields are added at the END and default to NULL. */
#ifndef GMFRONTEND_ITEMS_H
#define GMFRONTEND_ITEMS_H

typedef enum FrontendItemKind {
    FE_ACTION,
    FE_CHOICE,
    FE_SLIDER,
    FE_TOGGLE,
} FrontendItemKind;

typedef enum FrontendAction {
    FE_DO_CONTINUE, ///< go on to the mode the rule interrupted
    FE_DO_BACK,     ///< return to the mode the player came from
    FE_DO_CALL,     ///< run the item's `call` (the screen stays)
} FrontendAction;

typedef struct FrontendItem {
    unsigned char kind;   ///< ::FrontendItemKind
    unsigned char action; ///< ::FrontendAction, for FE_ACTION
    const char* label;
    const char* help;
    int (*get)(void);
    void (*set)(int);
    int min, max, step;
    const char* const* options;         ///< FE_CHOICE labels, index = value - min
    void (*format)(int value, char* out); ///< FE_SLIDER text; default "%d"
    int (*visible)(void);               ///< NULL = always shown
    void (*call)(void);                 ///< FE_DO_CALL
    int (*enabled)(void);               ///< NULL = enabled; a disabled row shows greyed, A bumps
    const char* off_reason;             ///< Atlas: said under a disabled row ("Connect a controller to this port first."); NULL = none
    const char* group;                  ///< Atlas: a heading above this row when it differs from the previous row's; NULL = none
} FrontendItem;

typedef struct FrontendScreen {
    const char* title;
    const char* subtitle;
    const FrontendItem* items;
    int n_items;
    int art; ///< a room screen drawn from its layout (gmfrontend_online.inc: FL_*), 0 = rows
} FrontendScreen;

#endif
