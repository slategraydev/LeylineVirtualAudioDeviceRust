# Makefile for Leyline APO
# This uses the WDK/SDK toolchain (cl.exe, link.exe, midl.exe)

PROJECT = LeylineAPO
DLL = $(PROJECT).dll
DEF = $(PROJECT).def
IDL = $(PROJECT).idl
HEADER = $(PROJECT)_h.h
SOURCES = LeylineAPO.cpp dllmain.cpp
OBJS = $(SOURCES:.cpp=.obj)

# Compiler and Linker flags
CXXFLAGS = /nologo /W4 /Zi /O2 /EHa /D_USRDLL /D_WINDLL /D_UNICODE /DUNICODE /I.
LDFLAGS = /nologo /dll /def:$(DEF) /incremental:no /opt:ref /opt:icf

# Build targets
all: $(DLL)

$(HEADER): $(IDL)
	midl /nologo /header $(HEADER) $(IDL)

$(OBJS): $(SOURCES) $(HEADER)
	cl $(CXXFLAGS) /c $(SOURCES)

$(DLL): $(OBJS) $(DEF)
	link $(LDFLAGS) /out:$(DLL) $(OBJS) user32.lib ole32.lib oleaut32.lib

clean:
	del /f /q $(DLL) $(OBJS) $(HEADER) $(PROJECT)_i.c $(PROJECT).tlb $(PROJECT).exp $(PROJECT).lib $(PROJECT).pdb
