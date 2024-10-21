vala_sources = [
  'main.vala',
  'application-window.vala',
  'server-page.vala'
]

dependencies = [
  gtk,
]

executable('lankm-gui', vala_sources,
  gresources_ui,
  dependencies: dependencies,
  vala_args: ['--pkg=linux', '--pkg=posix']
)
