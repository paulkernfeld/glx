![fixed demo](https://user-images.githubusercontent.com/5089629/35182178-96570232-fe0a-11e7-9822-e7cbe377b997.gif)


# LICENCE
I am not the creator(Of course). So please respect the original license of Cooper Hewitt which is SIL Open Font License.
https://github.com/cooperhewitt/cooperhewitt-typeface

# Why
https://github.com/cooperhewitt/cooperhewitt-typeface/issues/4

I think every Cooper Hewitt Windows user face the same problem. Here is the fix that I fixed with FontForge.

# In case you wanted to fix it yourself
You need to first install FontForge(It is free). There are a few things to fix for each original otf file.

 - `Font family name` in `PS Names`
 - `Weight Class` and `Style Map` in `OS\2`
 
 You can press `Ctrl + Shift + F` to open `Font Information` panel
 
 I set `Font family name` to the original `Font family name` plus its font weight behind it. You have to set it, otherwise when you install all 14 fonts you will get 7 only. I am not sure what is the problem but separating them by setting different font weight it fixes the problem.
 
 For `Weight Class` you should set select the corresponding one. For Heavy I choose `900 Black`.
 
 For `Style Map` you need to set it to `Italic` if it is Italic font.
 
 Once it was set, you can generate fonts by pressing `Ctrl + Shift + G` save to whatever font format you want.
 
 That's all. You can start with official font files to ensure files are proper and correct. If you trust me you can use files in here, if not you can fix them yourself.
 
