# Keylayout language

A tool for parsing keyboard layouts and emitting definitions for use by firmwares and other tools

It looks like this:

```
layout {
  5k 5k;
  5k 5k;
  5k 5k;
  2s [2] [0] [1] [8] [9] [7] 2s;
}

key scroll {
    out keyberon: "::keyberon::action::Action::Custom(super::CustomEvent::MouseScroll)";
}

key ml {
    out keyberon: "::keyberon::action::Action::Custom(super::CustomEvent::MouseLeft)";
}

key mr {
    out keyberon: "::keyberon::action::Action::Custom(super::CustomEvent::MouseRight)";
}

key ctrldown {
    out keyberon: "::keyberon::action::Action::MultipleKeyCodes(&[::keyberon::key_code::KeyCode::LCtrl, ::keyberon::key_code::KeyCode::Down].as_slice())";
}

key ctrlup {
    out keyberon: "::keyberon::action::Action::MultipleKeyCodes(&[::keyberon::key_code::KeyCode::LCtrl, ::keyberon::key_code::KeyCode::Up].as_slice())";
}

layer base {
  'q' >esc< 'w' 'e' 'r' 't' 'y' >bspace< 'u' >del< 'i' >'/'< 'o' >'\'< 'p';
  'a'@lshift 's' 'd' 'f' 'g' 'h' >'<'< 'j' >':'< 'k' >'>'< 'l'       ';'@rshift;
  'z'@lctrl  'x' 'c' 'v' 'b' 'n' >'"'< 'm' >'''< ',' >'_'< '.'       '/'@rctrl;
                lalt tab@lgui space@[sym] space@[num] enter@scroll ralt;
}

layer sym {
  '!' '@' '{' '}' '|' '`' >ml< '~' >mr< '\' n '"';
  '#'@lshift '$' '(' ')' n '+' '-' '/' '*' '''@rshift;
  '%'@lctrl '^' '[' ']' n '&' '=' ',' '.' '_'@rctrl;
      n lalt space  '=' n n;
}

layer num {
  '1' '2' '3' '4' '5' '6' >ml< '7' >mr< '8' '9' '0';
  f1@lshift f2 f3 f4 f5 left down up right volup@rshift;
  f6@lctrl f7 f8 f9 f10 pgdown ctrldown ctrlup pgup voldown@rctrl;
      n n '=' n n n;
}
```

I use it in my [keyboard firmware](https://github.com/simmsb/rusty-dilemma) to generate the [layout](https://github.com/simmsb/rusty-dilemma/blob/master/firmware/src/keys/layout.rs)
