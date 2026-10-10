"""Gradio's supported theme API; local system fonts and neutral colors only."""
import gradio as gr
from gradio.themes.utils.colors import Color

def theme():
    mono=Color(name='mono',c50='#fafafa',c100='#f5f5f5',c200='#e5e5e5',c300='#d4d4d4',
               c400='#a3a3a3',c500='#737373',c600='#525252',c700='#404040',c800='#262626',
               c900='#171717',c950='#0a0a0a')
    return gr.themes.Base(
        primary_hue=mono, secondary_hue=mono, neutral_hue=mono,
        font=['Segoe UI','Microsoft YaHei UI','system-ui','sans-serif'],
        font_mono=['Cascadia Code','Consolas','monospace'],
    ).set(
        body_background_fill='#fafafa',body_background_fill_dark='#111111',
        body_text_color='#171717',body_text_color_dark='#f5f5f5',
        body_text_color_subdued='#737373',body_text_color_subdued_dark='#a3a3a3',
        background_fill_primary='#ffffff',background_fill_primary_dark='#171717',
        background_fill_secondary='#f5f5f5',background_fill_secondary_dark='#202020',
        block_background_fill='#ffffff',block_background_fill_dark='#171717',
        block_border_color='#e5e5e5',block_border_color_dark='#333333',
        border_color_primary='#e5e5e5',border_color_primary_dark='#333333',
        border_color_accent='#171717',border_color_accent_dark='#eeeeee',
        border_color_accent_subdued='#d4d4d4',border_color_accent_subdued_dark='#404040',
        color_accent='#171717',color_accent_soft='#f5f5f5',color_accent_soft_dark='#262626',
        link_text_color='#171717',link_text_color_dark='#eeeeee',
        link_text_color_hover='#525252',link_text_color_hover_dark='#cccccc',
        input_background_fill='#ffffff',input_background_fill_dark='#202020',
        input_border_color='#d4d4d4',input_border_color_dark='#404040',
        input_border_color_focus='#171717',input_border_color_focus_dark='#eeeeee',
        input_shadow='none',input_shadow_focus='none',input_shadow_focus_dark='none',
        input_radius='10px',block_radius='12px',block_padding='18px',
        block_shadow='none',block_shadow_dark='none',
        checkbox_background_color_selected='#171717',checkbox_background_color_selected_dark='#f5f5f5',
        checkbox_border_color_selected='#171717',checkbox_border_color_selected_dark='#f5f5f5',
        checkbox_label_background_fill_selected='#f5f5f5',checkbox_label_background_fill_selected_dark='#262626',
        button_primary_background_fill='#171717',button_primary_background_fill_dark='#f5f5f5',
        button_primary_background_fill_hover='#333333',button_primary_background_fill_hover_dark='#d4d4d4',
        button_primary_text_color='#ffffff',button_primary_text_color_dark='#171717',
        button_primary_border_color='#171717',button_primary_border_color_dark='#f5f5f5',
        button_secondary_background_fill='#ffffff',button_secondary_background_fill_dark='#202020',
        button_secondary_background_fill_hover='#f5f5f5',button_secondary_background_fill_hover_dark='#333333',
        button_secondary_text_color='#171717',button_secondary_text_color_dark='#f5f5f5',
        button_secondary_border_color='#d4d4d4',button_secondary_border_color_dark='#404040',
        button_large_radius='10px',button_large_text_size='14px',button_large_text_weight='600',
        button_border_width='1px',button_border_width_dark='1px',
        button_primary_shadow='none',button_secondary_shadow='none',
        loader_color='#737373',loader_color_dark='#a3a3a3',
    )
